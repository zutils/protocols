use crate::transportresponse::TransportResponse;
use crate::{Transport, Destination, TransportToModuleGlue, ModuleToTransportGlue};
use crate::common::CommonModule;

use std::collections::HashMap;

#[derive(Default)]
/// If we want to be able to have multiple modules per plugin, we use this.
pub struct TransportNode {
    nodes: Vec<TransportNode>,
    modules: HashMap<String, Box<TransportToModuleGlue + Sync>>, // String is schema_url of some type... using Schema gave me errors.
}

impl TransportNode {
    pub fn add_interface<M: 'static + TransportToModuleGlue + CommonModule + Sync>(&mut self, module: M) {
        use crate::utils::AsStringer;
        match module.get_info(&Destination::new()) {
            Ok(ref vec_info) if vec_info.get_vec().len() > 1 => println!("Module is returning too much info!"),
            Ok(ref vec_info) if vec_info.get_vec().len() == 1 => {
                // If we have exactly one module_info, then we need to add that module's schema as a key
                let info = &vec_info.get_vec().to_vec()[0];
                let schema = info.get_schema();
                self.modules.insert(schema.as_string().to_string(), Box::new(module)); },
            Ok(ref vec_info) if vec_info.get_vec().len() == 0 => println!("No Info available from module!"),
            Ok(_) => println!("Cannot add module to Transport Node!"),
            Err(e) => println!("Cannot add module to Transport Node! {:?}", e),
        }
    }

    pub fn add_node(&mut self, node: TransportNode) {
        self.nodes.push(node);
    }

    pub fn handle_appropriate_modules(&self, transport: &Transport) -> Vec<Transport> {
        use crate::utils::AsStringer;

        let (transports, errors): (Vec<_>, Vec<_>) = self.modules.iter()
            .filter(|(schema, _)| !transport.has_destination() || *schema == transport.get_destination().as_string())
            .map(|(_, module)| module)
            .map(|module| module.handle_transport(transport) )
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
impl ModuleToTransportGlue for TransportNode {}

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
            .map(|node| node.propagate_transport(transport) )
            .flatten()
            .collect();
        ret.append(&mut new_transports);
        
        ret
    }
}
