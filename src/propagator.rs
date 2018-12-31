use crate::transportresponse::TransportResponse;
use crate::{Transport, Destination, TransportToModuleGlue, ModuleToTransportGlue};
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
        use crate::utils::AsStringer;
        match module.get_info(&Destination::new()) {
            Ok(ref vec_info) if vec_info.get_vec().len() > 1 => println!("Module is returning too much info!"),
            Ok(ref vec_info) if vec_info.get_vec().len() == 1 => {
                // If we have exactly one module_info, then we need to add that module's schema as a key
                let info = &vec_info.get_vec().to_vec()[0];
                let schema = info.get_schema();
                self.modules.insert(schema.as_string().to_string(), Arc::new(Mutex::new(module))); },
            Ok(ref vec_info) if vec_info.get_vec().len() == 0 => println!("No Info available from module!"),
            Ok(_) => println!("Cannot add module to Transport Node!"),
            Err(e) => println!("Cannot add module to Transport Node! {:?}", e),
        }
    }

    pub fn add_node<T: 'static + Propagator + Send + Sync>(&mut self, node: T) {
        self.nodes.push(Arc::new(RwLock::new(node)));
    }

    pub fn handle_appropriate_modules(&self, transport: &Transport) -> Vec<Transport> {
        use crate::utils::AsStringer;

        //println!("Attempting to handle {:?}", transport);

        let appropriate_modules = self.modules.iter()
            //.inspect(|(schema, _)| println!("Checking for appropriate schema: {:?}", schema) )
            .filter(|(schema, _)| !transport.has_destination() || *schema == transport.get_destination().as_string())
            .map(|(_, module)| module).collect::<Vec<_>>();

        if appropriate_modules.len() > 0 {
            println!("Found {} appropriate modules.", appropriate_modules.len());
        }

        let (transports, errors): (Vec<_>, Vec<_>) = appropriate_modules.iter()
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
