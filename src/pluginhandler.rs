//! The pluginhandler handles loading the correct plugins and routing calls between them.

#[cfg(not(target_arch = "wasm32"))]
use crate::ModuleToTransportGlue;

use crate::{Transport, VecTransport};
use crate::propagator::{Propagator, TransportNode};
use crate::transportresponse::TransportResponse;

#[cfg(not(target_arch = "wasm32"))]
pub struct PluginHandler {
    libraries: Vec<Box<crate::commonlibrary::CommonFFI>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PluginHandler {
    pub fn new() -> Self {
        PluginHandler{ libraries: Vec::new() }
    }

    /// Passthrough function
    pub fn load_and_cache_plugin(&mut self, path: &std::path::PathBuf) -> Result<(), failure::Error> {
        use crate::commonlibrary::PluginLoader;
        self.libraries.load_and_cache_plugin(&path)?;
        Ok(())
    }
}

pub fn ffi_handle_received_bytes(node: &TransportNode, bytes: &[u8]) -> Vec<u8> {
    let mut ret = VecTransport::default();

    match quick_protobuf::deserialize_from_slice::<Transport>(bytes) {
        Err(e) => {
            let transport = TransportResponse::create_Error(&format!("Cannot parse data! Possibly incorrect version. {:?}", e));
            ret.vec.push(transport);
        },
        Ok(transport) => {
            let mut vectransport_data: Vec<Transport> = node.propagate_transport(&transport);
            ret.vec.append(&mut vectransport_data);
        },
    } 

    // serialize_into_vec returns a result - one that we cannot pass back. Fail as gracefully as we can :(
    match quick_protobuf::serialize_into_vec(&ret) {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            log::error!("Cannot write VecTransport to bytes! {:?}", e);
            Vec::new() // Return NOTHING :( TODO: Write test case for this.
        }
    }
}

/// Allow us to use CommonModule functions on the PluginHandler
#[cfg(not(target_arch = "wasm32"))]
impl ModuleToTransportGlue for PluginHandler {}

/// We want to propagate over any dynamic library
#[cfg(not(target_arch = "wasm32"))]
impl Propagator for PluginHandler {
    fn propagate_transport(&self, transport: &Transport) -> Vec<Transport>  {
        let mut ret = Vec::new();
        //let libraries = self.libraries.lock();
        for lib in self.libraries.iter() {
            match (*lib).call_ffi_propagate(transport) {
                Ok(mut owned_transport) => ret.append(&mut owned_transport),
                Err(e) => log::error!("Error when propagating over dynamic library! {:?}", e),
            }
        }
        ret
    }
}
