//! The pluginmanager handles loading the correct plugins and routing calls between them.

extern crate notify;
extern crate libloading as lib;
extern crate signals;
extern crate glob;
extern crate failure;
extern crate serde;
extern crate serde_json;

use std::thread;
use std::ffi::OsString;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use self::failure::{Error, format_err};
use self::notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use self::signals::{Signal, Emitter, Am};

/// This structure handles standard function calls that all compatible dynamic libraries should support.
pub struct Plugin {
    library: lib::Library,
}

/// When we use the plugin directly, dereference to the hash
impl Deref for Plugin {
    type Target = lib::Library;

    fn deref(&self) -> &Self::Target {
        &self.library
    }
}

impl Plugin {

    /// Get a nice name for simple display. This function may be going away.
    pub fn get_name(&self) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = self.library.get(b"get_name")?;
            Ok(func())
        }
    }

    /// Handle all messages. Pass in PluginManager so that we can handle further submessage data
    pub fn handle(&self, manager: &PluginManager, data: &[u8]) -> Result<(), Error> {
        println!("Handling data...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&PluginManager, &[u8]) -> Result<(), Error>> = self.library.get(b"handle")?;
            func(manager, data)
        }
    }

    /// This function may be renamed to get_schema_url
    /// It exists to reference the schema that was used to create the library.
    pub fn get_schema_url(&self) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = self.library.get(b"get_hash")?;
            Ok(func())
        }
    }

    /// We want the protocol library itself to be responsible for generation of its own messages
    pub fn generate_message(&self, template_name: &str) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&str) -> Result<String, Error>> = self.library.get(b"generate_message")?;
            func(template_name)
        }
    }
}

/// The PluginManager exists to interface with the different plugins. It also has capabilities to dynamically modify those plugins.
pub struct PluginManager {
    map: Am<HashMap<String, Plugin>>,
}

impl PluginManager {
    /// Create a new PluginManager
    pub fn new() -> Self {
        PluginManager {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// If we know the schema hash, we can get a default message of the hash type.
    pub fn get_default_json_message(&self, hash: &str) -> Result<serde_json::Value, Error> {
        let default_msg = self.generate_message(hash, "")?; // No name is default.
        println!("Default Msg: {:?}", default_msg);
        Ok(serde_json::from_str(&default_msg)?)
    }

    /// Generate a string representation of the default message.
    pub fn generate_message(&self, hash: &str, template_name: &str) -> Result<String, Error> {
        let hash_map = self.map.lock().unwrap();
        hash_map.get(hash).ok_or(format_err!("Hash {} does not exist!", hash))?.generate_message(template_name)
    }

    /// Get the hashed library from the map, and call its exported "handle" function. 
    pub fn handle_msg_and_submsgs(&self, hash: &str, data: &[u8]) -> Result<(), Error> {
        println!("Attempting to handle message type: {} and data '{}'", hash, String::from_utf8(data.to_vec())?);

        let hash_map = self.map.lock().unwrap();
        let plugin = hash_map.get(hash).ok_or(format_err!("Hash {} does not exist!", hash))?;
        plugin.handle(self, data)
    }

    pub fn handle_msg_in_thread(&self, hash: &str, data: Vec<u8>) {
        let hash_map = self.map.clone();
        let hash = hash.to_string();
        thread::spawn(move || {
            let hash_map = hash_map.lock().unwrap();
            let plugin = hash_map.get(&hash).ok_or(format_err!("Hash {} does not exist!", hash))?;
            plugin.handle(self, &data)
        });
    }

    /// TODO: SUPPORT MORE THAN JUST WINDOWS DLL FILES!!!
    /// Default SHOULD be "./libraries/**/target/debug/*.dll" if libraries exist there.
    pub fn load_all_plugins(&self, path_glob: &str) -> Result<(), Error> {
        let hash_map = self.map.clone();
        let signal = Signal::new_arc_mutex(move |path: &OsString| PluginManager::load_plugin(hash_map.clone(), path));

        glob::glob(path_glob)?.filter_map(Result::ok).for_each(|path: PathBuf| {
            signal.lock().unwrap().emit(path.into_os_string());
        });

        Ok(())
    }

    fn blocking_watch_directory(watch_path: &str, on_path_changed: Am<Emitter<input=OsString>>) -> Result<(), Error> {
        let (transmit, receive) = channel();
        let mut watcher = raw_watcher(transmit).unwrap();
        watcher.watch(watch_path, RecursiveMode::Recursive)?;

        // Continuously loop and receive events
        loop {
            match receive.recv() {
                Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                    // Handle raw event
                    println!("Raw Event: {:?} {:?} {:?}", op, path, cookie);
                    match path.file_name().ok_or(format_err!(r#"Filename is "None""#)) {
                        Ok(os_str) => on_path_changed.lock().unwrap().emit(os_str.to_os_string()),
                        Err(e) => println!("{:?}", e),
                    }
                },
                Ok(event) => println!("Broken Directory Watcher Event: {:?}", event),
                Err(e) => println!("Directory Watcher Error: {:?}", e),
            }
        }
    }

    /// Load a single plugin given a filename
    pub fn load_single_plugin(&self, filename: &str) -> Result<(), Error> {
        PluginManager::load_plugin(self.map.clone(), &OsString::from(filename))
    }

    /// Loads a library so that we may use its functions
    fn load_plugin(hash_map: Am<HashMap<String,Plugin>>, filename: &OsString) -> Result<(), Error> {
        println!("Loading {:?}", filename);
        let library = lib::Library::new(filename)?;
    
        let plugin = Plugin {
            library
        };

        let hash = plugin.get_schema_url()?.to_string();
        println!("Loading plugin {:?} with hash {:?}", filename, hash);
        hash_map.lock().unwrap().insert(hash, plugin);
        Ok(())
    }

    /// Continuously load from plugin directories
    pub fn continuously_watch_for_new_plugins(&self, watch_path: &'static str) {
        let hash_map = self.map.clone();
        let signal = Signal::new_arc_mutex(move |path: &OsString| PluginManager::load_plugin(hash_map.clone(), path));

        thread::spawn(move || {           
            if let Err(e) = PluginManager::blocking_watch_directory(watch_path, signal ){
                println!("{:?}", e);
            }
        });
    }

}
