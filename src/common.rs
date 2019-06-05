use failure::Error;
use crate::autogen_protobuf::transport::*;
use hashbrown::HashMap;

pub trait CommonStructureFunctions {
    fn process(&mut self, data: &DataChanges) -> Result<DataChanges, Error>; 
}

pub trait CommonModelFunctions {
    fn constructor(&mut self, id: &Id) -> Result<VecDataChanges, Error>;
    fn destructor(&mut self, id: &Id) -> Result<VecDataChanges, Error>;
    fn update_data(&mut self, changes: &DataChanges) -> Result<VecDataChanges, Error>;
}

pub trait Modifiable {
    fn modify(&mut self, changes: &DataChanges) -> Result<Vec<DataChanges>, Error>;
    fn set_defaults(&mut self);
    fn get_all_data_changes(&self) -> Vec<DataChanges>;
}

pub struct ModelInterface<M: Default + Modifiable> {
    objects: HashMap<Id, M>,
}

impl<M> CommonModelFunctions for ModelInterface<M> where M: Default + Modifiable {
    /// Construct a new instance of the model.
    fn constructor(&mut self, id: &Id) -> Result<VecDataChanges, Error> {
        let mut obj = M::default(); // This will also create submodules.
        obj.set_defaults();

        // Remove old object if one exists.
        if let Some(_old_data) = self.objects.get(id) {
            self.destructor(id);
        }

        if let Some(_old_data) = self.objects.insert(id.clone(), obj) {
            log::warn!("Model {:?} has already been created! It should have been removed!", id);
        }

        Ok(VecDataChanges::new(obj.get_all_data_changes()))
    }

    /// Destroy and cleanup a model
    fn destructor(&mut self, id: &Id) -> Result<VecDataChanges, Error> {
        match self.objects.remove(&id) {
            None => log::warn!("Model {:?} does not exist to be removed!", id),
            Some(data) => {
                for data in data.get_all_data_changes() {
                    self.destructor(data.moduleId);
                }
            },
        }

        Ok(Empty::default())
    }

    /// Update the "data only"
    // If a submodel changes data based on this update_data, then those changes should be returned.
    // Furthermore, If we ARE a submodule of another model, we should return US AS changes.
        // I think this means that we must keep track of our parent.
    fn update_data(&mut self, changes: &DataChanges) -> Result<VecDataChanges, Error> {
        let obj = self.objects.get_mut(&changes.modelId).ok_or(failure::format_err!("Cannot update model. Missing {:?}", changes.modelId))?;
        let data_changes = obj.modify(&changes)?;
        Ok(VecDataChanges::new(data_changes))
    }
}
