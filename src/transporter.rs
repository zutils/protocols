use crate::autogen_protobuf::transport::*;
use crate::{ TransportToProcessorGlue, TransportToModelGlue };
use crate::{ CommonModelFunctions, CommonStructureFunctions, TransportResponse };

use failure::Error;

use hashbrown::HashMap;

#[derive(Default)]
/// If we want to be able to have multiple structures per plugin, we use this.
pub struct TransportNode {
    // Overloaded protobuf structure will just be pushed on the stack (for now)
    // By doing it this way, it is possible for a module to NOT let other modules handle it's functions.
    nodes: HashMap<ModuleId, Box<Transporter>>,
    struct_handlers: HashMap<ModuleId, Box<TransportToProcessorGlue>>, 
    model_handlers: HashMap<ModuleId, Box<TransportToModelGlue>>, 
}

impl TransportNode {
    // I'm not a fan of panics much, but these functions are pretty much initialization code!

    pub fn add_struct_handler<H: 'static + CommonStructureFunctions + Default>(&mut self, module_id: ModuleId) {
        let handler = Box::new(H::default());
        if let Some(_existing) = self.struct_handlers.insert(module_id.clone(), handler) {
            panic!("There already exists {:?} in transport node!", module_id); 
        }
    }

    pub fn add_model_handler<H: 'static + CommonModelFunctions + Default>(&mut self, module_id: ModuleId) {
        let handler = Box::new(H::default());
        if let Some(_existing) = self.model_handlers.insert(module_id.clone(), handler) {
            panic!("There already exists {:?} in transport node!", module_id); 
        }
    }

    pub fn add_node<T: 'static + Transporter>(&mut self, module_id: ModuleId, new_node: T) {
        if let Some(_existing) = self.nodes.insert(module_id.clone(), Box::new(new_node)) {
            panic!("There already exists {:?} in transport node!", module_id); 
        }
    }
}



/// For now, broadcast to ALL transport. We never know if there are duplicate plugins for the same schema
pub trait Transporter {
    fn transport_data(&mut self, transport: &RequestTransport) -> ReturnTransport;
}

impl Transporter for TransportNode {
    fn transport_data(&mut self, transport: &RequestTransport) -> ReturnTransport {
        let dest = &transport.moduleId;

        // Check handlers first
        if let Some(glue) = self.model_handlers.get_mut(&dest) {
            return match glue.handle_transport(transport) {
                Ok(ret) => ret,
                Err(e) => TransportResponse::create_TransportError(&format!("{:?}", e)), 
            };
        }
        if let Some(glue) = self.struct_handlers.get_mut(&dest) {
            return match glue.handle_transport(transport) {
                Ok(ret) => ret,
                Err(e) => TransportResponse::create_TransportError(&format!("{:?}", e)), 
            };
        }

        // Then check nodes.
        if let Some(node) = self.nodes.get_mut(&dest) {
            return node.transport_data(transport);
        }

        // If none exist, then just return an error
        TransportResponse::create_TransportError(&format!("Transporter does not have handler or node that supports {:?}", dest))
    }
}

pub struct RootTransporter {
    node: TransportNode,
    descriptor_to_module_ids: HashMap<TypeDescriptor, ModuleId>,
} 

impl RootTransporter {
    pub fn exec(&mut self, root_descriptor: &TypeDescriptor) -> Result<(), Error> {
        let root_module_id = uuid::Uuid::create_v4().into();
        let pending = VecId::new(Id::new(root_module_id, root_descriptor));

        // This will also update/create any data. Some data just keeps being sent through loops. Send that in the future.
        let mut leftovers = self.node.constructor(pending)?; 

        // When all data is created, 
        while true {
            let leftovers_clone = leftovers.clone();
            leftovers.clear();
            for leftover in leftovers_clone {
                leftovers = self.node.update_data(leftover);
            }

            leftovers += self.node.update_all_data();
        }
    }

    fn set_descriptor_module_id(&mut self, descriptor: TypeDescriptor, module_id: ModuleId) {
        self.descriptor_to_module_ids.insert(descriptor, module_id);
    }

    fn descriptor_to_module_id(&self, descriptor: &TypeDescriptor) -> Option<&ModuleId> {
        self.descriptor_to_module_ids.get(descriptor)
    }

    // Pass-through 
    pub fn add_struct_handler<H: 'static + CommonStructureFunctions + Default>(&mut self, descriptor: TypeDescriptor) {
        let module_id = uuid::Uuid::create_v4().into();
        self.node.add_struct_handler(module_id);
        self.set_descriptor_module_id(descriptor, module_id);
    }

    // Pass-through 
    pub fn add_model_handler<H: 'static + CommonModelFunctions + Default>(&mut self, descriptor: TypeDescriptor) {
        let module_id = uuid::Uuid::create_v4().into();
        self.node.add_model_handler(module_id);
        self.set_descriptor_module_id(descriptor, module_id);
    }


    pub fn constructor(&mut self, changes: DataChanges) -> Result<(), Error> {
        match self.descriptor_to_module_id(&changes.descriptor) {
            Some(module_id) => {
                let transport = TransportRequest::create_CONSTRUCTOR(module_id, changes.id);
                log::debug!("Calling constructor({:?})", transport);
                let pending_changes: VecDataChanges = self.transport_data(&transport).try_into()?;
                for change in pending_changes.vec {
                    if let Err(e) = self.constructor(change) {
                        log::warn!("{:?}", e);
                    }
                }
            },
            None => log::warn!("No module for descriptor {:?}!", changes.descriptor),
        }
        Ok(())
    }

    pub fn destructor(&mut self, changes: DataChanges) -> Result<VecId, Error> {
        match self.descriptor_to_module_id(&changes.descriptor) {
            Some(module_id) => {
                let transport = TransportRequest::create_DESTRUCTOR(module_id, changes.id);
                log::debug!("Calling destructor({:?})", transport);
                let pending_changes: VecDataChanges = self.transport_data(&transport).try_into()?;
                for change in pending_changes.vec {
                    if let Err(e) = self.destructor(change) {
                        log::warn!("{:?}", e);
                    }
                }
            },
            None => log::warn!("No module for descriptor {:?}!", changes.descriptor),
        }
        Ok(())
    }

    pub fn update_data(&mut self, changes: DataChanges) -> Result<VecDataChanges, Error> {
        match self.descriptor_to_module_id(&changes.descriptor) {
            Some(module_id) => {
                let transport = TransportRequest::create_UPDATE_DATA(module_id, changes.id);
                log::debug!("Calling update_data({:?})", transport);
                let pending_changes: VecDataChanges = self.transport_data(&transport).try_into()?;
                for change in pending_changes.vec {
                    if let Err(e) = self.update_data(change) {
                        log::warn!("{:?}", e);
                    }
                }
            },
            None => log::warn!("No module for descriptor {:?}!", changes.descriptor),
        }
        Ok(())
    }

    pub fn process(&mut self, data: DataChanges) -> Result<DataChanges, Error> {
        match self.descriptor_to_module_id(&data.descriptor) {
            Some(module_id) => {
                let transport = TransportRequest::create_PROCESS(module_id, data);
                log::debug!("Calling process({:?})", transport);
                let changes: DataChanges = self.transport_data(&transport).try_into()?;
                if let Err(e) = self.update_data(changes) {
                    log::warn!("{:?}", e);
                }
            },
            None => log::warn!("No module for descriptor {:?}!", data.descriptor),
        }
        Ok(())
    }
}
