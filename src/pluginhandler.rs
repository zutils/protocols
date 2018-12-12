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
        match self.to_string() {
            Ok(data) => write!(f, "{:?}", data),
            Err(e) => write!(f, "{:?}", e),
        }
    }
}

/// This structure handles standard function calls that all compatible dynamic libraries should support.
pub struct DynamicLibrary {
    library: lib::Library,
}

/// Structure to represent unhandled messages
#[derive(Debug)]
pub struct MessageInfo {
    pub schema_url: String,
    pub rpc_method_name: Option<String>,
    pub data: PluginData,
}

impl ::std::fmt::Display for MessageInfo {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "schema_url: {} method_name: {:?}, data: {}", self.schema_url, self.rpc_method_name, self.data)
    }
}

impl MessageInfo {
    pub fn new(schema_url: &str, data: &[u8]) -> Self {
        MessageInfo { schema_url: schema_url.to_string(), 
                      rpc_method_name: None,
                      data: PluginData(data.to_vec()), 
                    }
    }

    pub fn new_rpc(schema_url: &str, rpc_method_name: &str, data: &[u8]) -> Self {
        MessageInfo { schema_url: schema_url.to_string(), 
                      rpc_method_name: Some(rpc_method_name.to_string()),
                      data: PluginData(data.to_vec()), 
                    }
    }
}

#[derive(Debug)]
pub struct LibraryInfo {
    pub schema_url: String,
    pub name: String,
    pub ffi_version: String,
}

/// Each submessage in each library needs to impl these functions.
pub trait FFILibrary {
    /// We require a trait_version so that we can match it up with plugin version.
    fn get_trait_ffi_version(&self) -> &'static str { "0.0.7" } // Update this whenever we modify the FFILibrary trait.

    fn verify_plugin_and_trait_version(&self) -> Result<(), Error> {
        let trait_version = self.get_trait_ffi_version();
        for info in self.get_info()? {
            if info.ffi_version != trait_version {
                return Err(failure::format_err!("Plugin {:?} does not match trait version {:?}!", info, trait_version));
            }
        }
        Ok(())
    }


    /// Get any information about this library
    fn get_info(&self) -> Result<Vec<LibraryInfo>, Error>;

    /// Generate default message
    fn generate_default_message(&self, schema_url: &str, template: &str, args: Vec<&[u8]>) -> Result<PluginData, Error>;

    /// Handle all messages. Return a Vec<MessageInfo> so that we can handle further submessage data
    fn handle_trusted(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;

    /// Handle receiving of a trusted Remote Procedure Call
    fn receive_trusted_rpc(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;

    /// Handle receiving of an untrusted Remote Procedure Call
    fn receive_untrusted_rpc(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error>;
}


impl FFILibrary for DynamicLibrary {
    fn get_info(&self) -> Result<Vec<LibraryInfo>, Error> {
        println!("Plugin: Getting info...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn() -> Result<Vec<LibraryInfo>, Error>> = 
                        self.library.get(b"get_info").expect("get_info not found in library!");
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

    fn handle_trusted(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        println!("Plugin: Handling data...");
        unsafe {
            let func: lib::Symbol<unsafe extern fn(MessageInfo) -> Result<Vec<MessageInfo>, Error>> = 
                        self.library.get(b"handle").expect("handle function not found in library!");
            func(info)
        }
    }

    fn receive_trusted_rpc(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        println!("Plugin: Receiving trusted RPC for {:?}...", info);
        unsafe {
            let func: lib::Symbol<unsafe extern fn(MessageInfo) -> Result<Vec<MessageInfo>, Error>> = 
                        self.library.get(b"receive_trusted_rpc").expect("receive_trusted_rpc function not found in library!");
            func(info)
        }
    }

    fn receive_untrusted_rpc(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        println!("Plugin: Receiving untrusted RPC for {:?}...", info);
        unsafe {
            let func: lib::Symbol<unsafe extern fn(MessageInfo) -> Result<Vec<MessageInfo>, Error>> = 
                        self.library.get(b"receive_untrusted_rpc").expect("receive_untrusted_rpc function not found in library!");
            func(info)
        }
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

    /// Get the library from the map, and handle results as trusted
    pub fn handle_trusted_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        println!("Handle trusted message {:?}", info);
        let self_clone = self.clone();
        let plugin = self.get_plugin(&info.schema_url)?;

        // Call proper message
        let method_name = info.rpc_method_name.clone();
        let additional_messages = match method_name {
            Some(_method_name) => plugin.lock().unwrap().receive_trusted_rpc(info)?,
            None => plugin.lock().unwrap().handle_trusted(info)?,
        };

        // We want results to be handled non-blocking and iteratively.
        ::std::thread::spawn(move || {
            for msg in additional_messages.into_iter() {
                if let Err(e) = self_clone.handle_trusted_msg_and_submsgs(msg) {
                    println!("{:?}", e);
                }
            }
        });
        Ok(())
    }

    /// Get the library from the map, and handle results as untrusted
    pub fn handle_untrusted_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        println!("Handle untrusted message {:?}", info);
        let self_clone = self.clone();
        let plugin = self.get_plugin(&info.schema_url)?;

        // Call proper message
        let method_name = info.rpc_method_name.clone();
        let additional_messages = match method_name {
            Some(_method_name) => plugin.lock().unwrap().receive_untrusted_rpc(info)?,
            None => { println!("Cannot handle untrusted messages! Only Rpc!"); Vec::new() },
        };

        // We want results to be handled non-blocking and iteratively.
        ::std::thread::spawn(move || {
            for msg in additional_messages.into_iter() {
                if let Err(e) = self_clone.handle_untrusted_msg_and_submsgs(msg) {
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

        let infos = plugin.get_info()?;
        let plugin = Arc::new(Mutex::new(plugin));
        
        for info in infos {
            println!("Loading schema {:?} from plugin {:?}", info, path);
            self.lock().unwrap().insert(info.schema_url.clone(), plugin.clone());
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
