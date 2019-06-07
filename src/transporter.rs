use crate::autogen_protobuf::transport::*;
use crate::{ TransportToProcessorGlue, TransportToModelGlue };
use crate::{ CommonModelFunctions, CommonStructureFunctions };

use failure::Error;
use std::convert::TryInto;

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
                Err(e) => format!("{:?}", e).into(), 
            };
        }
        if let Some(glue) = self.struct_handlers.get_mut(&dest) {
            return match glue.handle_transport(transport) {
                Ok(ret) => ret,
                Err(e) => format!("{:?}", e).into(), 
            };
        }

        // Then check nodes.
        if let Some(node) = self.nodes.get_mut(&dest) {
            return node.transport_data(transport);
        }

        // If none exist, then just return an error
        format!("Transporter does not have handler or node that supports {:?}", dest).into()
    }
}

pub struct RootTransporter {
    node: TransportNode,
    descriptor_to_module_ids: HashMap<TypeDescriptor, ModuleId>,
} 

impl Transporter for RootTransporter {
    fn transport_data(&mut self, transport: &RequestTransport) -> ReturnTransport {
        self.node.transport_data(transport)
    }
}

impl RootTransporter {
    pub fn exec(&mut self, root_descriptor: TypeDescriptor) -> Result<(), Error> {
        let root_module_id = uuid::Uuid::new_v4();
        let root_construct = ConstructorData {
            id: Id::new(root_module_id.to_string()),
            descriptor: root_descriptor,
            ..Default::default()
        };

        // This will also update/create any data. Some data just keeps being sent through loops. Send that in the future.
        let mut events = self.constructor(root_construct)?; 

        // This is the runtime loop!
        let mut done = false;
        while !done {
            let mut new_events = Vec::new();
            for event in events {
                match self.handle_event(event) {
                    Err(e) => log::warn!("{:?}", e),
                    Ok(mut new) => new_events.append(&mut new),
                }
            }
            
            events = new_events;
            if events.len() == 0 { done = true; }
        }

        println!("No more events. Quitting!");
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<Vec<Event>, Error> {
        match event.data {
            mod_Event::OneOfdata::constructor(data) => self.constructor(data),
            mod_Event::OneOfdata::destructor(data) => self.destructor(data),
            mod_Event::OneOfdata::update_model(data) => self.update_model(data),
            mod_Event::OneOfdata::process_struct(data) => self.process_struct(data),
            mod_Event::OneOfdata::None => Err(failure::format_err!("Event type is None!")),
        }
    }

    fn set_descriptor_module_id(&mut self, descriptor: TypeDescriptor, module_id: ModuleId) {
        self.descriptor_to_module_ids.insert(descriptor, module_id);
    }

    fn descriptor_to_module_id(&self, descriptor: &TypeDescriptor) -> Result<ModuleId, Error> {
        match self.descriptor_to_module_ids.get(descriptor) {
            Some(id) => Ok(id.clone()),
            None => Err(failure::format_err!("No module for descriptor {:?}!", descriptor)),
        }
    }

    // Pass-through 
    pub fn add_struct_handler<H: 'static + CommonStructureFunctions + Default>(&mut self, descriptor: TypeDescriptor) {
        let module_id = ModuleId::new(uuid::Uuid::new_v4().to_string());
        self.node.add_struct_handler::<H>(module_id.clone());
        self.set_descriptor_module_id(descriptor, module_id);
    }

    // Pass-through 
    pub fn add_model_handler<H: 'static + CommonModelFunctions + Default>(&mut self, descriptor: TypeDescriptor) {
        let module_id = ModuleId::new(uuid::Uuid::new_v4().to_string());
        self.node.add_model_handler::<H>(module_id.clone());
        self.set_descriptor_module_id(descriptor, module_id);
    }

    fn transport(&mut self, descriptor: &TypeDescriptor, data: mod_Event::OneOfdata) -> Result<Vec<Event>, Error> {
        let module_id = self.descriptor_to_module_id(&descriptor)?;
        let transport = RequestTransport::new(module_id, Event::new(data));
        let ret = self.transport_data(&transport).try_into()?;
        Ok(ret)
    }
}

impl CommonStructureFunctions for RootTransporter {
    /// Update structures only!!! When a structure is updated, return those structures for updating elsewhere.
    fn process_struct(&mut self, data: ProcessStructData) -> Result<Vec<Event>, Error> {
        log::debug!("Calling process_struct({:?})...", data);
        let ret = self.transport(&data.changes.descriptor.clone(), data.into())?;
        log::debug!("...Returned from process_struct(...)");
        Ok(ret)
    }
}

impl CommonModelFunctions for RootTransporter {
    /// This is how you create your root object. It will return any objects we need to manually create.
    fn constructor(&mut self, data: ConstructorData) -> Result<Vec<Event>, Error> {
        log::debug!("Calling constructor({:?})...", data);
        let ret = self.transport(&data.descriptor.clone(), data.into())?;
        log::debug!("...Returned from constructor(...)");
        Ok(ret)
    }

    /// Destroys an object.  Recurse by destroying any returned sub-objects
    fn destructor(&mut self, data: DestructorData) -> Result<Vec<Event>, Error> {
        log::debug!("Calling destructor({:?})...", data);
        let ret = self.transport(&data.descriptor.clone(), data.into())?;
        log::debug!("...Returned from destructor(...)");
        Ok(ret)
    }

    /// Updates models only!!!. It returns any model changes or structure changes necessary for the next go-around.
    fn update_model(&mut self, data: UpdateModelData) -> Result<Vec<Event>, Error> {
        log::debug!("Calling update_model({:?})...", data);
        let ret = self.transport(&data.changes.changes.descriptor.clone(), data.into())?;
        log::debug!("...Returned from update_model(...)");
        Ok(ret)
    }
}
