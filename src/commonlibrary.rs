#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use failure::Error;
use crate::Transport;

pub trait CommonFFI {
    fn call_ffi_propagate(&self, transport: &Transport) -> Result<Vec<Transport>, Error>;
    fn call_ffi_init(&self) -> Result<(), Error>;
}

#[cfg(not(target_arch = "wasm32"))]
impl CommonFFI for libloading::Library {
    // TODO: Handle c-style ffi
    fn call_ffi_propagate(&self, transport: &Transport) -> Result<Vec<crate::Transport>, Error> {
        log::trace!("Calling FFI function 'propagate_ffi(...)'...");

        let bytes = quick_protobuf::serialize_into_vec(transport)?;

        let from_ffi = unsafe {
            let propagate: libloading::Symbol<unsafe extern fn(&[u8]) -> Vec<u8>> = self.get(b"propagate_ffi")?;
            propagate(&bytes)
        };

        let ret: crate::VecTransport = quick_protobuf::deserialize_from_slice(&from_ffi)?;
        log::trace!("...Received from FFI: {:?}", ret);
        Ok(ret.vec)
    }

    fn call_ffi_init(&self) -> Result<(), Error> {
        log::debug!("Calling FFI function 'init()'...");
        unsafe {
            let init: libloading::Symbol<unsafe extern fn()> = self.get(b"init")?;
            init();
        }
        log::debug!("...init() successful!");
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub trait PluginLoader {
    fn load_and_cache_plugin(&mut self, path: &PathBuf) -> Result<(), Error> {
        if !path.exists() {
            return Err(failure::format_err!("Failed to load dynamic library. {:?} does not exist!", path));
        }

        let ext = path.extension().ok_or(failure::format_err!("Cannot determine extension for {:?}", path))?;
        if ext == "wasm" {
            self.load_and_cache_webasm(path)
        } else {
            self.load_and_cache_dll(path)
        }
    }

    fn load_dll(&self, path: &PathBuf) -> Result<Box<CommonFFI>, Error> {
        if !path.exists() {
            return Err(failure::format_err!("Failed to load dynamic library. {:?} does not exist!", path));
        }

        log::debug!("Loading dynamic library {:?}...", path);
        let library = libloading::Library::new(path)?;
        library.call_ffi_init()?;
        log::debug!("...{:?} loaded successfully.", path);
        Ok(Box::new(library))
    }

    fn load_webasm(&self, path: &PathBuf) -> Result<Box<CommonFFI>, Error> {
        if !path.exists() {
            return Err(failure::format_err!("Failed to load wasm library. {:?} does not exist!", path));
        }

        log::debug!("Loading webasm library {:?}...", path);
        let mut library = crate::wasmhandler::WasmModule::new(path.clone());
        library.load()?;
        library.call_ffi_init()?;
        log::debug!("...{:?} loaded successfully.", path);
        Ok(Box::new(library))
    }

    fn load_and_cache_dll(&mut self, path: &PathBuf) -> Result<(), Error>;
    fn load_and_cache_webasm(&mut self, path: &PathBuf) -> Result<(), Error>;
}

#[cfg(not(target_arch = "wasm32"))]
impl PluginLoader for Vec<Box<CommonFFI>> {
    fn load_and_cache_dll(&mut self, path: &PathBuf) -> Result<(), Error> {
        let plugin = self.load_dll(path)?;
        self.push(plugin);
        Ok(())
    }

    fn load_and_cache_webasm(&mut self, path: &PathBuf) -> Result<(), Error> {
        let plugin = self.load_webasm(path)?;
        self.push(plugin);
        Ok(())
    }
}
