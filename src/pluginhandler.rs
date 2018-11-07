//! The pluginhandler handles loading the correct plugins and routing calls between them.

extern crate notify;
extern crate libloading as lib;
extern crate signals;
extern crate glob;
extern crate failure;
extern crate serde;
extern crate serde_json;

use std::thread;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use self::failure::{Error, format_err};
use self::notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use self::signals::{Signal, Emitter, Am};

/// This structure handles standard function calls that all compatible dynamic libraries should support.
pub struct DynamicLibrary {
    library: lib::Library,
}

/// When we use the plugin directly, dereference to the library
impl Deref for DynamicLibrary {
    type Target = lib::Library;

    fn deref(&self) -> &Self::Target {
        &self.library
    }
}

/// Structure to represent unhandled messages
#[derive(Debug)]
pub struct MessageInfo {
    pub history: Vec<String>,
    pub schema_url: String,
    pub data: Vec<u8>,
}

impl MessageInfo {
    pub fn new(history: Vec<String>, schema_url: &str, data: &[u8]) -> Self {
        MessageInfo { history, schema_url: schema_url.to_string(), data: data.to_vec() }
    }
}

pub trait FFILibrary {
    /// Get a nice name for simple display. This function may be going away.
    fn get_name(&self) -> Result<String, Error>;

    /// Handle all messages. Return a Vec<MessageInfo> so that we can handle further submessage data
    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;

    /// It exists to reference the schema that was used to create the library.
    fn get_schema_url(&self) -> Result<String, Error>;

    /// We want the protocol library itself to be responsible for generation of its own messages
    fn generate_message(&self, template_name: &str) -> Result<String, Error>;

    /// We will use this for plugins, but will return None on static plugins.MessageInfo
    fn get_library(&self) -> Result<&lib::Library, Error>;
}

impl FFILibrary for DynamicLibrary {
    fn get_name(&self) -> Result<String, Error> {
        println!("Plugin: Getting name...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = 
                        self.library.get(b"get_name").expect("get_name not found in library!");
            Ok(func())
        }
    }

    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        println!("Plugin: Handling data...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn(MessageInfo) -> Result<Vec<MessageInfo>, Error>> = 
                        self.library.get(b"handle").expect("handle function not found in library!");
            func(info)
        }
    }

    fn get_schema_url(&self) -> Result<String, Error> {
        println!("Plugin: Get Schema URL...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = 
                        self.library.get(b"get_schema_url").expect("get_schema_url function not found in library!");
            Ok(func())
        }
    }

    fn generate_message(&self, template_name: &str) -> Result<String, Error> {
        println!("Plugin: Generating message {}...", template_name);
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&str) -> Result<String, Error>> = 
                        self.library.get(b"generate_message").expect("generate_message function not found in library!");
            func(template_name)
        }
    }

    fn get_library(&self) -> Result<&lib::Library, Error> {
        Ok(&self.library)
    }
}

pub type FFILibraryHashMapValue = Am<FFILibrary+Send>;
type FFILibraryHashMap = HashMap<String, FFILibraryHashMapValue>; 

pub struct PluginHandler {
    map: Am<FFILibraryHashMap>,
}

/// When we use the plugin directly, dereference to the hashmap
impl Deref for PluginHandler {
    type Target = Am<FFILibraryHashMap>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Clone for PluginHandler {
    fn clone(&self) -> PluginHandler { 
        PluginHandler { map: self.map.clone() }
    }
}

impl PluginHandler {
    /// Create a new PluginHandler
    pub fn new() -> Self {
        PluginHandler {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_plugin(&self, schema: &str) -> Result<FFILibraryHashMapValue, Error> {
        let err = format_err!("Schema {} does not exist!", schema);
        let map = self.map.lock().unwrap();
        let plugin = map.get(schema).ok_or(err)?;
        Ok(plugin.clone())
    }

    fn get_plugin_from_info(&self, info: &MessageInfo) -> Result<FFILibraryHashMapValue, Error> {
        Ok(self.get_plugin(&info.schema_url)?)
    }

    /// Get the library from the map, and call its exported "handle" function. 
    pub fn handle_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        println!("Handle message {:?}", info);
        let self_clone = self.clone();
        let plugin = self.get_plugin_from_info(&info)?;
        let submsgs = plugin.lock().unwrap().handle(info)?;
        thread::spawn(move || {
            for msg in submsgs.into_iter() {
                if let Err(e) = self_clone.handle_msg_and_submsgs(msg) {
                    println!("{:?}", e);
                }
            }
        });
        Ok(())
    }

    /// Take a path glob and load in all plugins in that glob
    pub fn load_all_plugins(&self, path_glob: &str) -> Result<(), Error> {
        let hash_map = self.clone();
        let signal = Signal::new_arc_mutex(move |path: &PathBuf| hash_map.load_plugin(path));

        glob::glob(path_glob)?.filter_map(Result::ok).for_each(|path: PathBuf| {
            signal.lock().unwrap().emit(path);
        });

        Ok(())
    }

    /// So that you can load different plugins while the application is running.
    pub fn load_plugin(&self, path: &PathBuf) -> Result<String, Error> {
        let path = path.to_str().ok_or(format_err!("Cannot convert to string"))?;
        println!("Loading {}", path);
        let library = lib::Library::new(path)?;
    
        let plugin = DynamicLibrary {
            library
        };

        let schema = plugin.get_schema_url()?.to_string();
        println!("Loading plugin {:?} with schema {:?}", path, schema);
        self.lock().unwrap().insert(schema.clone(), Arc::new(Mutex::new(plugin)));
        Ok(schema)
    }

    /// Continuously load from plugin directories
    pub fn continuously_watch_for_new_plugins(&self, watch_path: PathBuf) {
        let hash_map = self.clone();
        let signal = Signal::new_arc_mutex(move |path: &PathBuf| hash_map.load_plugin(path));

        thread::spawn(move || {           
            if let Err(e) = blocking_watch_directory(watch_path, signal ){
                println!("{:?}", e);
            }
        });
    }

}

// Start file watcher on watch_path. Emit on_path_changed if a file changes.
fn blocking_watch_directory(watch_path: PathBuf, on_path_changed: Am<Emitter<input=PathBuf>>) -> Result<(), Error> {
    let (transmit, receive) = channel();
    let mut watcher = raw_watcher(transmit).unwrap();
    watcher.watch(watch_path, RecursiveMode::Recursive)?;

    // Continuously loop and receive events
    loop {
        match receive.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                println!("Raw Event: {:?} {:?} {:?}", op, path, cookie);
                on_path_changed.lock().unwrap().emit(path);
            },
            Ok(event) => println!("Broken Directory Watcher Event: {:?}", event),
            Err(e) => println!("Directory Watcher Error: {:?}", e),
        }
    }
}