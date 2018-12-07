//! The pluginhandler handles loading the correct plugins and routing calls between them.

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use libloading as lib;
use failure::{Error, format_err};
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use signals::{Signal, Emitter, Am};

/// Any message data being passed around will use PluginData
#[derive(Debug, Clone)]
pub struct PluginData(pub Vec<u8>);

impl PluginData {
    pub fn as_ref(&self) -> &[u8] { &self.0 }
    pub fn as_str(&self) -> Result<&str, Error> { Ok(::std::str::from_utf8(&self.0)?) }
    pub fn to_string(&self) -> Result<String, Error> { Ok(self.as_str()?.to_string()) }
}

impl ::std::fmt::Display for PluginData {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

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
    pub schema_url: String,
    pub data: PluginData,
}

impl MessageInfo {
    pub fn new(schema_url: &str, data: &[u8]) -> Self {
        MessageInfo { schema_url: schema_url.to_string(), 
                      data: PluginData(data.to_vec()), 
                    }
    }
}

/// Each submessage in each library needs to impl these functions.
pub trait SubLibrary {
    /// Get a nice name for simple display. This function may be going away.
    fn get_name(&self) -> String;

    /// Handle all messages. Return a Vec<MessageInfo> so that we can handle further submessage data
    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;

    // "where Self: Sized" is because we don't take &self. https://doc.rust-lang.org/error-index.html#method-has-no-receiver
    /// It exists to reference the schema that was used to create the library.
    fn get_schema_url() -> String where Self: Sized;

    /// Generate default message
    fn generate_default_message(&self, template: &str, args: Vec<&[u8]>) -> Result<PluginData, Error>;

    /// Handle receiving of a Remote Procedure Call
    fn receive_rpc(&self, data: &[u8]) -> Result<(), Error>;
}

pub trait FFILibrary {
    /// We require a trait_version so that we can match it up with plugin version.
    fn get_trait_ffi_version(&self) -> &'static str { "0.0.4" } // Update this whenever we modify the FFILibrary trait.

    fn verify_plugin_and_trait_version(&self) -> Result<(), Error> {
        let trait_version = self.get_trait_ffi_version();
        let plugin_version = &self.get_plugin_ffi_version()?;
        if plugin_version != trait_version {
            return Err(failure::format_err!("Plugin version {:?} does not match trait version {:?}!", plugin_version, trait_version));
        }
        Ok(())
    }

    /// Get name of sublibrary
    fn get_name(&self, schema_url: &str) -> Result<String, Error>;

    /// Handle all messages. Return a Vec<MessageInfo> so that we can handle further submessage data
    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;

    /// Return a list of all schema urls
    fn get_schema_urls(&self) -> Result<Vec<String>, Error>;

    /// Get the FFI version fromt he plugin.
    fn get_plugin_ffi_version(&self) -> Result<String, Error>;

    /// We will use this for plugins, but will return None on static plugins.MessageInfo
    fn get_library(&self) -> Result<&lib::Library, Error>;

    /// Generate a default message for a named plugin
    fn generate_default_message(&self, schema_url: &str, template: &str, args: Vec<&[u8]>) -> Result<PluginData, Error>;

    /// Handle receiving of a Remote Procedure Call
    fn receive_rpc(&self, schema_url: &str, data: &[u8]) -> Result<(), Error>;
}

impl FFILibrary for DynamicLibrary {
    fn get_name(&self, schema_url: &str) -> Result<String, Error> {
        println!("Plugin: Getting name...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&str) -> Result<String, Error>> = 
                        self.library.get(b"get_name").expect("get_name not found in library!");
            func(schema_url)
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

    fn get_schema_urls(&self) -> Result<Vec<String>, Error> {
        println!("Plugin: Get Schema URL...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> Result<Vec<String>, Error>> = 
                        self.library.get(b"get_schema_urls").expect("get_schema_urls function not found in library!");
            func()
        }
    }

    fn get_plugin_ffi_version(&self) -> Result<String, Error> {
        println!("Plugin: Get FFI Version...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> Result<String, Error>> = 
                        self.library.get(b"get_plugin_ffi_version").expect("get_plugin_ffi_version function not found in library!");
            func()
        }
    }

    fn generate_default_message(&self, schema_url: &str, template: &str, args: Vec<&[u8]>) -> Result<PluginData, Error> {
        println!("Plugin: Generate default message {:?}...", schema_url);
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&str, &str, Vec<&[u8]>) -> Result<PluginData, Error>> = 
                        self.library.get(b"generate_default_message").expect("generate_default_message function not found in library!");
            func(schema_url, template, args)
        }
    }

    fn receive_rpc(&self, schema_url: &str, data: &[u8]) -> Result<(), Error> {
        println!("Plugin: Receiving RPC for {:?}...", schema_url);
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&str, &[u8]) -> Result<(), Error>> = 
                        self.library.get(b"receive_rpc").expect("receive_rpc function not found in library!");
            func(schema_url, data)
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
        ::std::thread::spawn(move || {
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
    pub fn load_plugin(&self, path: &PathBuf) -> Result<(), Error> {
        let path_str = path.to_str().ok_or(format_err!("Cannot convert to string"))?;
        println!("Current Dir: {:?}", std::env::current_dir()?);
        println!("Loading {}", path_str);

        if !path.exists() {
            println!("Path {:?} does not exist!", path);
        }

        let library = lib::Library::new(path)?;
    
        let plugin = DynamicLibrary {
            library
        };

        plugin.verify_plugin_and_trait_version()?;

        println!("{:?} loaded successfully.", path);

        let schema_urls: Vec<String> = plugin.get_schema_urls()?;
        let am_plugin = Arc::new(Mutex::new(plugin));
        
        for url in schema_urls.iter() {
            println!("Loading schema {} from plugin {:?}", url, path);
            self.lock().unwrap().insert(url.clone(), am_plugin.clone());
        }
        Ok(())
    }

    /// Continuously load from plugin directories
    pub fn continuously_watch_for_new_plugins(&self, watch_path: PathBuf) {
        let hash_map = self.clone();
        let signal = Signal::new_arc_mutex(move |path: &PathBuf| hash_map.load_plugin(path));

        ::std::thread::spawn(move || {           
            if let Err(e) = blocking_watch_directory(watch_path, signal ){
                println!("{:?}", e);
            }
        });
    }

}

// Start file watcher on watch_path. Emit on_path_changed if a file changes.
fn blocking_watch_directory(watch_path: PathBuf, on_path_changed: Am<Emitter<input=PathBuf>>) -> Result<(), Error> {
    use std::sync::mpsc::channel;

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
