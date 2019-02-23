use crate::transportresponse::TransportResponse;
use crate::{ModuleInfo, Transport, Destination, TransportToModuleGlue, ModuleToTransportGlue};
use crate::common::CommonModule;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Default)]
/// If we want to be able to have multiple modules per plugin, we use this.
/// Note that each module requires a lock to use it. The propogation itself should only be locked when adding modules.
pub struct TransportNode {
    nodes: Vec<Arc<RwLock<Propagator + Send + Sync>>>,
    modules: HashMap<String, Arc<Mutex<TransportToModuleGlue + Send>>>, // String is schema_url of some type... using Schema gave me errors.
}

impl TransportNode {
    pub fn add_interface<M: 'static + TransportToModuleGlue + CommonModule + Send>(&mut self, module: M) {
        match module.get_info(&Destination::default()) {
            Ok(ref vec_info) if vec_info.vec.len() > 1 => log::warn!("Module is returning too much info!"),
            Ok(ref mut vec_info) if vec_info.vec.len() == 1 => {
                // If we have exactly one module_info, then we need to add that module's schema as a key
                let info: ModuleInfo = vec_info.vec.pop().unwrap_or(ModuleInfo::default());
                self.modules.insert(info.schema.val, Arc::new(Mutex::new(module))); },
            Ok(ref vec_info) if vec_info.vec.len() == 0 => log::warn!("No Info available from module!"),
            Ok(_) => log::error!("Cannot add module to Transport Node!"),
            Err(e) => log::error!("Cannot add module to Transport Node! {:?}", e),
        }
    }

    pub fn add_node<T: 'static + Propagator + Send + Sync>(&mut self, node: T) {
        self.nodes.push(Arc::new(RwLock::new(node)));
    }

    pub fn get_appropriate_modules(&self, transport: &Transport) -> Vec<&String> {
        // ALL modules are allowed if there is no destination
        if let None = transport.destination {
            log::debug!("No Destination found. Using all modules.");
            return self.modules.iter().map(|(key, _)| key).collect();
        }

        if let Some(schema_dest) = &transport.destination {
            let schema_dest = &schema_dest.val;
            return self.modules.iter()
                .inspect(|(schema, _)| log::trace!("Checking for appropriate schema: {:?}", schema) )
                .filter(move |(schema, _)| **schema == *schema_dest)
                .map(|(key, _)| key).collect::<Vec<_>>();
        }

        Vec::new() // None
    }

    /// Handle modules that is have the proper destination. If there is no destination, handle everywhere.
    pub fn handle_appropriate_modules(&self, transport: &Transport) -> Vec<Transport> {
        log::trace!("Attempting to handle {:?}", transport);

        let appropriate_modules = self.get_appropriate_modules(transport);
        if appropriate_modules.len() > 0 {
            log::debug!("Found {} appropriate modules.", appropriate_modules.len());
        }

        let (transports, errors): (Vec<_>, Vec<_>) = appropriate_modules.iter()
            .map(|&key| &self.modules[key])
            .map(|module| module.lock().unwrap().handle_transport(transport) )
            .partition(Result::is_ok);

        let mut transports: Vec<_> = transports.into_iter().map(Result::unwrap).flatten().collect();

        // Create transports out of errors
        let mut errors: Vec<_> = errors.into_iter().map(Result::unwrap_err)
            .map(|e| format!("Module Error: {:?}", e))
            .map(|e| TransportResponse::create_Error(&e))
            .collect();
        
        transports.append(&mut errors);
        transports
    }
}

// Allow being able to call module functions on node
impl<'a> ModuleToTransportGlue for TransportNode {}

/// For now, broadcast to ALL transport. We never know if there are duplicate plugins for the same schema
pub trait Propagator {
    fn propagate_transport(&self, transport: &Transport) -> Vec<Transport>;
}

impl Propagator for TransportNode {
    /// If the current node has the schema we are looking for, call it's respective module. Otherwise, send to all leaf nodes
    fn propagate_transport(&self, transport: &Transport) -> Vec<Transport> {
        let mut ret = self.handle_appropriate_modules(transport);
        
        // Propagate and collect.
        let mut new_transports: Vec<Transport> = self.nodes.iter()
            .map(|node| node.read().unwrap().propagate_transport(transport) )
            .flatten()
            .collect();
        ret.append(&mut new_transports);
        
        ret
    }
}
