//! The pluginhandler handles loading the correct plugins and routing calls between them.

#[cfg(not(target_arch = "wasm32"))]
use crate::{ Transporter, TransportResponse};
use crate::autogen_protobuf::transport::*;

use hashbrown::HashMap;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct PluginHandler {
    libraries: HashMap<ModuleId, Box<crate::commonlibrary::CommonFFI>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl PluginHandler {
    /// Passthrough function
    pub fn load_and_cache_plugin(&mut self, path: &std::path::PathBuf) -> Result<(), failure::Error> {
        use crate::commonlibrary::PluginLoader;
        self.libraries.load_and_cache_plugin(&path)?;
        Ok(())
    }
}

pub fn ffi_handle_received_bytes<T: Transporter>(transporter: &mut T, bytes: &[u8]) -> Vec<u8> {
    let transport = match quick_protobuf::deserialize_from_slice::<RequestTransport>(bytes) {
        Err(e) => TransportResponse::create_TransportError(&format!("Cannot parse data! Possibly incorrect version. {:?}", e)),
        Ok(transport) => transporter.transport_data(&transport),
    };

    // serialize_into_vec returns a result - one that we cannot pass back. Fail as gracefully as we can :(
    match quick_protobuf::serialize_into_vec(&transport) {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            log::error!("Cannot write ReturnTransport to bytes! {:?}", e);
            Vec::new() // Return NOTHING :( TODO: Write test case for this.
        }
    }
}

/// We want to propagate over any dynamic library
#[cfg(not(target_arch = "wasm32"))]
impl Transporter for PluginHandler {
    fn transport_data(&mut self, transport: &RequestTransport) -> ReturnTransport { 
        let dest = &transport.moduleId;
        if let Some(node) = self.libraries.get(&dest) {
            return match node.call_ffi_handle_request(transport) {
                Ok(ret) => ret,
                Err(e) => TransportResponse::create_TransportError(&format!("Return Transport Error: {:?}", e)),
            }
        };

        // If none exist, then just return an error
        TransportResponse::create_TransportError(&format!("PluginHandler does not have handler or node that supports {:?}", dest))
    }
}
