//! The pluginhandler handles loading the correct plugins and routing calls between them.

use std::path::PathBuf;

use failure::Error;
use notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};
use signals::{Signal, Emitter, Am};

use crate::transmission_interface::transmission::Error as TError;
use crate::transmission_interface::transmission::{self, Transmission, Schema, Data, DataType, VecTransmission};
use crate::transmission_interface::temp_transmission_rpc::ModuleToTransportationGlue;
use crate::transport::Propagator;

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
    fn call_ffi_propagate(&self, transmission: &Transmission) -> Result<Vec<Transmission>, Error>;
}

impl CommonFFI for libloading::Library {
    // TODO: Handle c-style ffi
    fn call_ffi_propagate(&self, transmission: &Transmission) -> Result<Vec<Transmission>, Error> {
        use protobuf::Message;

        println!("Calling FFI function 'propagate_ffi(...)'");

        let bytes = transmission.write_to_bytes()?;

        let from_ffi = unsafe {
            let propagate: libloading::Symbol<unsafe extern fn(&[u8]) -> Vec<u8>> = self.get(b"propagate_ffi")?;
                    //.expect("propagate_ffi([u8]) function not found in library!"));
            propagate(&bytes)
        };

        let ret: transmission::VecTransmission = protobuf::parse_from_bytes(&from_ffi)?;
        Ok(ret.vec.into_vec())
    }
}

pub trait ToDataConverter: protobuf::Message {
    fn to_data(&self, ipfs: &str) -> Result<Data, Error> {
        let mut schema = Schema::new();
        schema.set_Ipfs(ipfs.to_string());
        let serialized_data = self.write_to_bytes()?;

        let mut ret = Data::new();
        ret.set_decode_schema(schema);
        ret.set_serialized_data(serialized_data);
        Ok(ret)
    }
}

fn from_data<T: protobuf::Message>(data: &Data) -> Result<(Schema, T), Error> {
    let schema = data.get_decode_schema();
    let serialized_data = data.get_serialized_data();
    Ok((schema.to_owned(), protobuf::parse_from_bytes(serialized_data)?))
}

pub fn create_error_transmission(error: &str) -> Transmission {
    let mut ret = Transmission::new();
    let mut data_type = DataType::new();
    let mut t_error = TError::new();
    t_error.set_error(error.to_string());
    data_type.set_error(t_error);
    ret.set_payload(data_type);
    ret
}

pub fn parse_data_as_transmission(data: &[u8]) -> Result<Transmission, Error> {
    Ok(protobuf::parse_from_bytes(&data)?)
}

trait VecTransmissionTranslater {
    fn to_VecTransmission(vec_transmission: Vec<Transmission>) -> VecTransmission;
}

impl VecTransmissionTranslater for Vec<Transmission> {
    fn to_VecTransmission(vec_transmission: Vec<Transmission>) -> VecTransmission {
        let mut ret = VecTransmission::new();
        for t in vec_transmission {
            ret.vec.push(t);
        }
        ret
    }
}

/// Allow us to use CommonModule functions on the PluginHandler
impl ModuleToTransportationGlue for PluginHandler {}

/// We want to propagate over any dynamic library
impl Propagator for PluginHandler {
    fn propagate_transmission(&self, transmission: &Transmission) -> Vec<Transmission> {
        let mut ret = Vec::new();
        let libraries = self.libraries.lock();
        for lib in libraries.iter() {
            match lib.call_ffi_propagate(transmission) {
                Ok(mut transmissions) => ret.append(&mut transmissions),
                Err(e) => println!("Error when propagating over dynamic library! {:?}", e),
            }
        }
        ret
    }
}


/*pub type CommonModuleHashMapValue = Am<CommonModule+Send>;
type CommonModuleHashMap = HashMap<String, CommonModuleHashMapValue>; 

pub struct PluginHandler {
    map: Am<CommonModuleHashMap>,
}

/// When we use the plugin directly, dereference to the hashmap
impl Deref for PluginHandler {
    type Target = Am<CommonModuleHashMap>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Clone for PluginHandler {
    fn clone(&self) -> PluginHandler { 
        PluginHandler { map: self.map.clone() }
    }
}*/

/*impl PluginHandler {
    /// Create a new PluginHandler
    pub fn new() -> Self {
        PluginHandler {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_plugin(&self, schema: &str) -> Result<CommonModuleHashMapValue, Error> {
        let err = format_err!("Schema {} does not exist!", schema);
        let map = self.map.lock().unwrap();
        let plugin = map.get(schema).ok_or(err)?;
        Ok(plugin.clone())
    }

    /// Get the library from the map, and handle results as trusted
    pub fn handle_trusted_msg_and_submsgs(&self, info: MessageInfo) -> Result<(), Error> {
        println!("Handle trusted message {:?}", info);
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
        let plugin = self.get_plugin(&info.schema_link)?;

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
                println!("{:?}", e);
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
    println!("Current Dir: {:?}", std::env::current_dir()?);
    println!("Loading {}", path_str);

    if !path.exists() {
        println!("Path {:?} does not exist!", path);
    }

    let library = libloading::Library::new(path)?;
    println!("{:?} loaded successfully.", path);
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
                println!("Raw Event: {:?} {:?} {:?}", op, path, cookie);
                on_path_changed.lock().emit(path);
            },
            Ok(event) => println!("Broken Directory Watcher Event: {:?}", event),
            Err(e) => println!("Directory Watcher Error: {:?}", e),
        }
    }
}
