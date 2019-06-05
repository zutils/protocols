
use std::path::PathBuf;
use hashbrown::HashMap;
use failure::Error;

use crate::{ RequestTransport, ReturnTransport };

/*#[cfg(not(target_arch = "wasm32"))]
impl From<PathBuf> for WasmModule {
    fn from(path: PathBuf) -> Self {
        WasmModule::new(path)
    }
}*/

#[cfg(not(target_arch = "wasm32"))]
pub struct WasmModule{
    path: PathBuf,
    instance: Option<wasmer_runtime::Instance>,
}

use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref RETURN_IDENTIFIER: Mutex<HashMap<i32, Vec<u8>>> = { std::sync::Mutex::new(HashMap::new()) };
}

lazy_static::lazy_static! {
    static ref ARGS_IDENTIFIER: Mutex<HashMap<i32, Vec<u8>>> = { std::sync::Mutex::new(HashMap::new()) };
}

#[cfg(not(target_arch = "wasm32"))]
impl WasmModule {
    pub fn new(path: PathBuf) -> Self {
        WasmModule{ path, instance: None}
    }

    pub fn load(&mut self) -> Result<(), Error> {
        use std::io::Read;
        
        let mut wasi_import_object = wasmer_wasi::generate_import_object(vec![], vec![], vec![]);

        let import_object = wasmer_runtime::imports! {
            "env" => {
                "return_data" => wasmer_runtime::func!(return_data),
                "get_arg_data" => wasmer_runtime::func!(get_arg_data),
                "test" => wasmer_runtime::func!(test),
            },
        };

        wasi_import_object.extend(import_object);

        // Read wasm file as bytes
        let mut wasm_data: Vec<u8> = Vec::new();
        let mut file = std::fs::File::open(&self.path)?;
        file.read_to_end(&mut wasm_data)?;

        log::debug!("Instantiating wasm...");
        self.instance = match wasmer_runtime::instantiate(&wasm_data, &wasi_import_object) {
            Ok(instance) => Some(instance),
            Err(e) => {
                println!("Error with instantiate: {:?}", e);
                return Err(failure::format_err!("{:?}", e))
            },
        };

        log::debug!("...finished loading wasm.");
        Ok(())
    }

    fn write_bytes_to_static_var(&self, identifier: i32, bytes: &[u8]) {
        let mut identifiers = ARGS_IDENTIFIER.lock().unwrap();
        if identifiers.contains_key(&identifier) {
            log::warn!("Identifier is already in the arg data! Module should use a random new identifier or pull the data!");
        } else {
            identifiers.insert(identifier, bytes.to_vec());
        }
    }

    fn invoke_with_hacky_bytes_arg(&self, func_name: &str, bytes: &[u8]) -> Result<Vec<wasmer_runtime::Value>, Error> {
        let id = rand::random::<i32>();
        self.write_bytes_to_static_var(id, bytes);
        let size = bytes.len();
        let args = vec![wasmer_runtime::Value::I32(id), wasmer_runtime::Value::I32(size as _)];
        Ok(self.invoke(func_name, &args)?)
    }

    fn invoke(&self, func_name: &str, args: &[wasmer_runtime::Value]) -> Result<Vec<wasmer_runtime::Value>, Error> {
        let instance = match &self.instance {
            None => return Err(failure::format_err!("Wasm Module not initialzed!")),
            Some(instance) => instance,
        };

        let ret = match instance.call(func_name, args) {
            Ok(ret) => Ok(ret),
            Err(e) => Err(failure::format_err!("{:?}", e)),
        };
        ret
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl crate::commonlibrary::CommonFFI for WasmModule {
    fn call_ffi_handle_request(&self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        log::trace!("Calling wasm FFI function 'handle_request_ffi(...)'...");

        let bytes = quick_protobuf::serialize_into_vec(transport)?;
        let results = self.invoke_with_hacky_bytes_arg("handle_request_ffi_wasm", &bytes)?;
        // Results is an identifier of what is to be returned.
        // The module could send back a DIFFERENT identifier of another module (if it can guess it)
        // If this happens, then it can trick unsafe data paths when safe modules return!
        // TODO WARNING!: This NEEDS TO BE FIXED BEFORE RELEASE!
        let identifier = match results.get(0) {
            Some(wasmer_runtime::Value::I32(identifier)) => identifier,
            Some(other) => return Err(failure::format_err!("call_ffi_handle_request did not return an i32! Found {:?}", other)),
            None => return Err(failure::format_err!("call_ffi_handle_request did not return anything! Expecting an i32!")),
        };

        let from_ffi = RETURN_IDENTIFIER.lock().unwrap().remove(identifier)
            .ok_or(failure::format_err!("return_data result does not exist in map for call_ffi_handle_request!"))?;

        // Read back from UNIQUE_CALLS.
        let ret: ReturnTransport = quick_protobuf::deserialize_from_slice(&from_ffi)?;
        log::trace!("...Received from FFI: {:?}", ret);
        Ok(ret)
    }

    fn call_ffi_init(&self) -> Result<(), Error> {
        log::debug!("Calling wasm FFI function 'init()'...");
        let _result = self.invoke("init", &[])?;
        log::debug!("...init() successful!");
        Ok(())
    }
}

// Webasm calls this function to return byte data
#[cfg(not(target_arch = "wasm32"))]
fn return_data(ctx: &mut wasmer_runtime::Ctx, ptr: u32, len: u32, identifier: i32) {
    log::info!("Returned data from webasm!");
    let memory = ctx.memory(0);
    let ptr = ptr as usize;
    let len = len as usize;
    let view = &memory.view()[ptr..(ptr+len)];
    let bytes: Vec<_> = view.iter().map(|cell| cell.get()).collect();

    // TODO: This is a weak system - it should be replaced when wasmer supports it!
    let mut identifiers = RETURN_IDENTIFIER.lock().unwrap();
    if identifiers.contains_key(&identifier) {
        log::warn!("Identifier is already in the return data! Module should use a random new identifier! Transport data ignored!");
    } else {
        identifiers.insert(identifier, bytes);
    }
}

// Webasm calls this function to retrieve byte data
// TODO: This is a weak system - it should be replaced when wasmer supports it!
#[cfg(not(target_arch = "wasm32"))]
fn get_arg_data(ctx: &mut wasmer_runtime::Ctx, ptr: u32, len: u32, identifier: i32) {
    log::info!("Getting data for webasm!");
    println!("Getting data for webasm!");

    let arg = match ARGS_IDENTIFIER.lock().unwrap().remove(&identifier) {
        Some(arg) => arg,
        None => {
            log::warn!("get_arg does not exist with identifier {:?} !", identifier);
            return;
        },
    };

    let ptr = ptr as usize;
    let len = len as usize;

    if len != arg.len() {
        log::warn!("get_arg argument of size {:?} differs from requested size of {:?}!", arg.len(), len);
        println!("get_arg argument of size {:?} differs from requested size of {:?}!", arg.len(), len);
        return;
    }
        
    let memory = ctx.memory(0);
    for (i, byte) in arg.iter().enumerate() {
        memory.view()[ptr+i].set(*byte);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn test(ctx: &mut wasmer_runtime::Ctx) {
    println!("In Test!");
}