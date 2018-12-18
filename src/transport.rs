//use crate::transmission_interface::transmission::mod_Schema;
use crate::transmission_interface::transmission::Schema_oneof_data;
use crate::transmission_interface::transmission::{Schema, Transmission, Empty};
use crate::transmission_interface::temp_transmission_rpc::{ModuleToTransportationGlue, TransportationToModuleGlue, CommonModule};
use std::collections::HashMap;

#[derive(Default)]
pub struct TransportationNode {
    nodes: Vec<TransportationNode>,
    modules: HashMap<String, Box<TransportationToModuleGlue>>, // String is schema_url of some type... using Schema gave me errors.
}

impl TransportationNode {
    pub fn add_interface<M: 'static + TransportationToModuleGlue + CommonModule>(&mut self, module: M) {
        match module.get_info(Empty::new()) {
            Ok(ref vec_info) if vec_info.get_vec().len() > 1 => println!("Module is returning too much info!"),
            Ok(ref vec_info) if vec_info.get_vec().len() == 1 => {
                let info = &vec_info.get_vec().to_vec()[0];
                let schema = info.get_schema();
                self.modules.insert(schema.as_string().to_string(), Box::new(module)); },
            Ok(ref vec_info) if vec_info.get_vec().len() == 0 => println!("No Info available from module!"),
            Ok(_) => println!("Cannot add module to Transportation Node!"),
            Err(e) => println!("Cannot add module to Transportation Node! {:?}", e),
        }
    }

    pub fn add_node(&mut self, node: TransportationNode) {
        self.nodes.push(node);
    }
}

// Allow being able to call module functions on node
impl ModuleToTransportationGlue for TransportationNode {}

/// For now, broadcast to ALL transmission. We never know if there are duplicate plugins for the same schema
/// TODO: Handling of those duplicates is a task in itself.
/// TODO: Returning transmissions can be poisoned via webasm unsafe code and can be a security risk if not handled properly!!!
pub trait Propagator {
    fn propagate_transmission(&self, transmission: &Transmission) -> Vec<Transmission>;
}

impl Propagator for TransportationNode {
    /// If the current node has the schema we are looking for, call it's respective module. Otherwise, send to all leaf nodes
    fn propagate_transmission(&self, transmission: &Transmission) -> Vec<Transmission> {
        let mut ret = Vec::new();
        
        // Handle if there is no destination.
        if transmission.destination.is_none() {
            let mut new_transmissions = self.modules.iter()
                .fold(Vec::new(), |_all, (_key, module)| module.handle_transmission(transmission) );
            ret.append(&mut new_transmissions);
        } else { 
            // If we are going to a destination, append the transmissions
            let dest_schema = transmission.get_destination();
            for (schema, module) in self.modules.iter() {
                if schema == dest_schema.as_string() {
                    let mut new_transmissions = module.handle_transmission(transmission);
                    ret.append(&mut new_transmissions);
                }
            }
        }
        
        // Propagate and collect.
        let mut new_transmissions = self.nodes.iter()
            .fold(Vec::new(), |_all, node| node.propagate_transmission(transmission) );
        ret.append(&mut new_transmissions);
        
        ret
    }
}

pub trait AsStringer {
    fn as_string(&self) -> &str;
}

impl AsStringer for Schema {
    fn as_string(&self) -> &str {
        match self.data {
            Some(Schema_oneof_data::URL(ref m)) => m,
            Some(Schema_oneof_data::Ipfs(ref m)) => m,
            Some(Schema_oneof_data::Ipns(ref m)) => m,
            None => "",
        } 
    }
}

