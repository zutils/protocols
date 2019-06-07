use failure::Error;
use crate::autogen_protobuf::transport::*;
use hashbrown::HashMap;

pub trait CommonStructureFunctions {
    fn process_struct(&mut self, data: ProcessStructData) -> Result<Vec<Event>, Error>; 
}

pub trait CommonModelFunctions {
    fn constructor(&mut self, data: ConstructorData) -> Result<Vec<Event>, Error>;
    fn destructor(&mut self, data: DestructorData) -> Result<Vec<Event>, Error>;
    fn update_model(&mut self, data: UpdateModelData) -> Result<Vec<Event>, Error>;
}

pub trait Modifiable {
    fn modify(&mut self, changes: &ModelDataChanges);
    fn set_defaults(&mut self);
    fn get_all_model_changes(&self) -> Vec<ModelDataChanges>;
    fn get_all_struct_changes(&self) -> Vec<StructDataChanges>;
    fn get_all_sub_object_ids(&self) -> Vec<(Id, TypeDescriptor)>;
}

pub struct ModelInterface<M: Default + Modifiable> {
    objects: HashMap<Id, M>,
}

/// This is a standard model interface. All models will have a hashmap of objects. 
/// They will all support the trait Modifiable - which functions will be auto-generated.
impl<M> CommonModelFunctions for ModelInterface<M> where M: Default + Modifiable {
    /// Construct a new instance of the model.
    fn constructor(&mut self, data: ConstructorData) -> Result<Vec<Event>, Error> {
        let mut obj = M::default(); // This will also create submodules.
        obj.set_defaults();

        // Fail if old object exists.
        if let Some(_old_data) = self.objects.get(&data.id) {
            return Err(failure::format_err!("Old data exists for: {:?}", data));
        }

        let events: Vec<Event> = obj.get_all_model_changes().iter()
            .map(|changes| Event::new(ConstructorData{
                id: changes.id.clone(),
                descriptor: changes.changes.descriptor.clone(),
                serializedData: Some(changes.changes.serializedData.clone()),
            }.into())).collect();

        if let Some(_old_data) = self.objects.insert(data.id.clone(), obj) {
            log::warn!("Model {:?} has already been created! It should have been removed!... wierd.", data);
        }

        Ok(events)
    }

    /// Destroy and cleanup a model
    fn destructor(&mut self, data: DestructorData) -> Result<Vec<Event>, Error> {
        let obj = match self.objects.remove(&data.id) {
            None => return Err(failure::format_err!("Model {:?} does not exist to be removed!", data)),
            Some(obj) => obj,
        };

        let events: Vec<Event> = obj.get_all_sub_object_ids().iter()
            .map(|(id, descriptor)| Event::new(DestructorData{
                id: id.clone(),
                descriptor: descriptor.clone(),
            }.into())).collect();
        Ok(events)
    }

    /// Update the "data only". Return any events that are necessary due to the sideeffects of update_model(...)
    fn update_model(&mut self, data: UpdateModelData) -> Result<Vec<Event>, Error> {
        let obj = self.objects.get_mut(&data.id).ok_or(failure::format_err!("Cannot update model. Missing {:?}", data))?;
        obj.modify(&data.changes);
        let model_change_events: Vec<Event> = obj.get_all_model_changes().iter()
            .map(|changes| Event::new(UpdateModelData{
                id: changes.id.clone(),
                changes: changes.clone(),
            }.into())).collect();

        let mut struct_change_events: Vec<Event> = obj.get_all_struct_changes().iter()
            .map(|changes| Event::new(ProcessStructData{
                changes: changes.clone(),
            }.into())).collect();
            
        let mut events = model_change_events;
        events.append(&mut struct_change_events);
        Ok(events)
    }
}
