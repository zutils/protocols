//! Libraries are loaded from the hard drive. There are known "safe" libraries that are compiled with this crate.
//! It is possible to load unsafe libraries

extern crate notify;
extern crate libloading as lib;
extern crate signals;
extern crate glob;
extern crate failure;

use std::thread;
use std::ffi::OsString;
use std::sync::mpsc::channel;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use self::failure::{Error};
use self::notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use self::signals::{Signal, Emitter, Am};


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
    pub fn get_name(&self) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = self.library.get(b"get_name")?;
            Ok(func())
        }
    }

    pub fn handle(&self, data: &[u8]) -> Result<(), Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn(&[u8]) -> Result<(), Error>> = self.library.get(b"handle")?;
            func(data)
        }
    }

    pub fn get_hash(&self) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> String> = self.library.get(b"get_hash")?;
            Ok(func())
        }
    }

    pub fn get_default_message(&self) -> Result<String, Error> {
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> Result<String, Error>> = self.library.get(b"get_default_message")?;
            func()
        }
    }
}

pub struct PluginManager {
    map: Am<HashMap<String, Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// If we know the schema hash, we can get a default message of the hash type.
    pub fn get_default_message(&self, hash: &str) -> Result<String, Error> {
        let hash_map = self.map.lock().unwrap();
        hash_map.get(hash).ok_or(format_err!("Hash {} does not exist!", hash))?.get_default_message()
    }

    /// Get the hashed library from the map, and call its "handle" function
    pub fn handle(&self, hash: &str, data: &[u8]) -> Result<(), Error> {
        let hash_map = self.map.lock().unwrap();
        hash_map.get(hash).ok_or(format_err!("Hash {} does not exist!", hash))?.handle(data)
    }

    /// TODO: SUPPORT MORE THAN JUST WINDOWS DLL FILES!!!
    pub fn load_all_plugins(&self) -> Result<(), Error> {
        let hash_map = self.map.clone();
        let signal = Signal::new_arc_mutex(move |path: &OsString| PluginManager::load_plugin(hash_map.clone(), path));

        glob::glob("./libraries/**/target/debug/*.dll")?.filter_map(Result::ok).for_each(|path: PathBuf| {
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

        let hash = plugin.get_hash()?.to_string();
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
