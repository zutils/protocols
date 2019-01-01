//! The pluginhandler handles loading the correct plugins and routing calls between them.

use std::path::PathBuf;

use failure::Error;
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use signals::{Signal, Emitter, Am};
use protobuf::Message;

use crate::{Transport, VecTransport, ModuleToTransportGlue};
use crate::propagator::{Propagator, TransportNode};
use crate::transportresponse::TransportResponse;

//#[derive(Default)]
pub struct PluginHandler {
    libraries: Am<Vec<libloading::Library>>,
}

impl PluginHandler {
    pub fn new() -> Self {
        PluginHandler{ libraries: Am::new(Vec::new()), }
    }
}

impl DynamicLibraryLoader for PluginHandler {
    fn get_library_list(&self) -> Am<Vec<libloading::Library>> {
        self.libraries.clone()
    }
}

trait CommonFFI {
    fn call_ffi_propagate(&self, transport: &Transport) -> Result<Vec<Transport>, Error>;
}

impl CommonFFI for libloading::Library {
    // TODO: Handle c-style ffi
    fn call_ffi_propagate(&self, transport: &Transport) -> Result<Vec<Transport>, Error> {
        use protobuf::Message;

        log::debug!("Calling FFI function 'propagate_ffi(...)'");

        let bytes = transport.write_to_bytes()?;

        let from_ffi = unsafe {
            let propagate: libloading::Symbol<unsafe extern fn(&[u8]) -> Vec<u8>> = self.get(b"propagate_ffi")?;
            propagate(&bytes)
        };

        let ret: VecTransport = protobuf::parse_from_bytes(&from_ffi)?;
        log::trace!("Received from FFI: {:?}", ret);
        Ok(ret.vec.into_vec())
    }
}

pub fn ffi_handle_received_bytes(node: &TransportNode, bytes: &[u8]) -> Vec<u8> {
    let mut ret = VecTransport::new();

    match protobuf::parse_from_bytes(bytes) {
        Err(e) => {
            let transport = TransportResponse::create_Error(&format!("Cannot parse data! Possibly incorrect version. {:?}", e));
            ret.vec.push(transport);
        },
        Ok(transport) => {
            let vectransport_data = node.propagate_transport(&transport);
            ret.vec = protobuf::RepeatedField::from_vec(vectransport_data);
        },
    } 

    // write_to_bytes returns a result - one that we cannot pass back. Fail as gracefully as we can :(
    match ret.write_to_bytes() {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            log::error!("Cannot write VecTransport to bytes! {:?}", e);
            Vec::new() // Return NOTHING :( TODO: Write test case for this.
        }
    }
}

/// Allow us to use CommonModule functions on the PluginHandler
impl ModuleToTransportGlue for PluginHandler {}

/// We want to propagate over any dynamic library
impl Propagator for PluginHandler {
    fn propagate_transport(&self, transport: &Transport) -> Vec<Transport> {
        let mut ret = Vec::new();
        let libraries = self.libraries.lock();
        for lib in libraries.iter() {
            match lib.call_ffi_propagate(transport) {
                Ok(mut transports) => ret.append(&mut transports),
                Err(e) => log::error!("Error when propagating over dynamic library! {:?}", e),
            }
        }
        ret
    }
}



/*
    /// Get the library from the map, and handle results as trusted
    pub fn handle_trusted_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        log::debug!("Handle trusted message {:?}", info);
        let self_clone = self.clone();
        let plugin = self.get_plugin(&info.schema_link)?;

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
                    log::debug!("{:?}", e);
                }
            }
        });
        Ok(())
    }

    /// Get the library from the map, and handle results as untrusted
    pub fn handle_untrusted_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        log::debug!("Handle untrusted message {:?}", info);
        let self_clone = self.clone();
        let plugin = self.get_plugin(&info.schema_link)?;

        // Call proper message
        let method_name = info.rpc_method_name.clone();
        let additional_messages = match method_name {
            Some(_method_name) => plugin.lock().unwrap().receive_untrusted_rpc(info)?,
            None => { log::debug!("Cannot handle untrusted messages! Only Rpc!"); Vec::new() },
        };

        // We want results to be handled non-blocking and iteratively.
        ::std::thread::spawn(move || {
            for msg in additional_messages.into_iter() {
                if let Err(e) = self_clone.handle_untrusted_msg_and_submsgs(msg) {
                    log::debug!("{:?}", e);
                }
            }
        });
        Ok(())
    }
}*/

pub trait DynamicLibraryLoader {
    fn get_library_list(&self) -> Am<Vec<libloading::Library>>;

    /// Take a path glob and load in all plugins in that glob
    fn load_all_plugins(&self, path_glob: &str) -> Result<(), Error> {
        let library = self.get_library_list();
        let emitter = Signal::new_arc_mutex(move |path: PathBuf| {
            let new_plugin = load_plugin(&path)?;
            library.lock().push(new_plugin);
            Ok(())
        });

        glob::glob(path_glob)?.filter_map(Result::ok).for_each(|path: PathBuf| {
            emitter.lock().emit(path);
        });

        Ok(())
    }

    /// Continuously load from plugin directories
    fn continuously_watch_for_new_plugins(&self, watch_path: PathBuf) {
        let library = self.get_library_list();
        let emitter = Signal::new_arc_mutex(move |path: PathBuf| {
            let new_plugin = load_plugin(&path)?;
            library.lock().push(new_plugin);
            Ok(())
        });

        ::std::thread::spawn(move || {           
            if let Err(e) = blocking_watch_directory(watch_path, emitter ){
                log::error!("{:?}", e);
            }
        });
    }

    fn load_plugin(&self, path: &PathBuf) -> Result<(), Error> {
        let new_plugin = load_plugin(&path)?;
        self.get_library_list().lock().push(new_plugin);
        Ok(())
    }
}

/// So that you can load different plugins while the application is running.
fn load_plugin(path: &PathBuf) -> Result<libloading::Library, Error> {
    let path_str = path.to_str().ok_or(failure::format_err!("Cannot convert to string"))?;
    log::debug!("Current Dir: {:?}", std::env::current_dir()?);
    log::info!("Loading {}", path_str);

    if !path.exists() {
        log::warn!("Path {:?} does not exist!", path);
    }

    let library = libloading::Library::new(path)?;
    log::info!("{:?} loaded successfully.", path);
    Ok(library)
}

// Start file watcher on watch_path. Emit on_path_changed if a file changes.
fn blocking_watch_directory<E>(watch_path: PathBuf, on_path_changed: Am<E>) -> Result<(), Error> 
    where E: Emitter<input=PathBuf>
{
    use std::sync::mpsc::channel;

    let (transmit, receive) = channel();
    let mut watcher = raw_watcher(transmit).unwrap();
    watcher.watch(watch_path, RecursiveMode::Recursive)?;

    // Continuously loop and receive events
    loop {
        match receive.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(op), cookie}) => {
                log::debug!("Raw Event: {:?} {:?} {:?}", op, path, cookie);
                on_path_changed.lock().emit(path);
            },
            Ok(event) => log::error!("Broken Directory Watcher Event: {:?}", event),
            Err(e) => log::error!("Directory Watcher Error: {:?}", e),
        }
    }
}
