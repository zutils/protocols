#![allow(non_snake_case)]

use crate::transporter::{Transporter, RootTransporter};
use crate::transportresponse::TransportResponse;
use crate::common::*;
use crate::autogen_protobuf::transport::*;

use std::convert::TryInto;
use failure::Error;

// Anywhere there is a common model or structure, we want transports working with them.
impl<T> TransportToModelGlue for T where T: CommonModelFunctions {}
impl<T> TransportToProcessorGlue for T where T: CommonStructureFunctions {}

/// These functions are the endpoints to the different modules.
pub trait TransportToProcessorGlue: CommonStructureFunctions {
    fn handle_transport(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        // Pass on transport to proper function
        match transport.requestFunctionType {
            RequestFunctionType::PROCESS => self.process_glue(transport),
            other => return Err(failure::format_err!("{:?} request function type unsupported!", other)),
        }
    }

    fn process_glue(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        if let mod_RequestData::OneOfdata::datachanges(msg) = &transport.payload.data {
            let module_ret = self.process(&msg)?;
            Ok(TransportResponse::create(module_ret.into()))
        } else {
            Err(failure::format_err!("Improper data type for update_data(DataChanges) message!"))
        }
    }
}

/// These functions are the endpoints to the different modules.
pub trait TransportToModelGlue: CommonModelFunctions {
    fn handle_transport(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        // Pass on transport to proper function
        match transport.requestFunctionType {
            RequestFunctionType::CONSTRUCTOR => self.constructor_glue(transport),
            RequestFunctionType::DESTRUCTOR => self.destructor_glue(transport),
            RequestFunctionType::UPDATE_DATA => self.update_data_glue(transport),
            other => return Err(failure::format_err!("{:?} request function type unsupported!", other)),
        }
    }

    fn constructor_glue(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        if let mod_RequestData::OneOfdata::datachanges(msg) = &transport.payload.data {
            let module_ret = self.constructor(&msg)?;
            Ok(TransportResponse::create(module_ret.into()))
        } else {
            Err(failure::format_err!("Improper data type for constructor(DataChanges) message!"))
        }
    }

    fn destructor_glue(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        if let mod_RequestData::OneOfdata::datachanges(msg) = &transport.payload.data {
            let module_ret = self.destructor(&msg)?;
            Ok(TransportResponse::create(module_ret.into()))
        } else {
            Err(failure::format_err!("Improper data type for destructor(DataChanges) message!"))
        }
    }

    fn update_data_glue(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> { 
        if let mod_RequestData::OneOfdata::datachanges(msg) = &transport.payload.data {
            let module_ret = self.update_data(&msg)?;
            Ok(TransportResponse::create(module_ret.into()))
        } else {
            Err(failure::format_err!("Improper data type for update_data(DataChanges) message!"))
        }
    }
}

/// This is glue to package up requests into Transports and unpacking them.
pub trait ModelFunctionToTransportGlue: RootTransporter {
    fn constructor(&mut self, changes: DataChanges) -> Result<(), Error> {
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

    fn destructor(&mut self, changes: DataChanges) -> Result<VecId, Error> {
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

    fn update_data(&mut self, changes: DataChanges) -> Result<VecDataChanges, Error> {
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
}

/// This is glue to package up requests into Transports and unpacking them.
pub trait StructureFunctionToTransportGlue: Transporter {
    fn process(&mut self, data: DataChanges) -> Result<DataChanges, Error> {
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

#[derive(Default)]
pub struct TransportRequest;
impl TransportRequest {
    pub fn create_CONSTRUCTOR(module_id: &ModuleId, data: DataChanges) -> RequestTransport {
        RequestTransport {
            moduleId: module_id.clone(),
            payload: RequestData::new(mod_RequestData::OneOfdata::datachanges(data)),
            requestFunctionType: RequestFunctionType::CONSTRUCTOR,
        }
    }

    pub fn create_DESTRUCTOR(module_id: &ModuleId, data: DataChanges) -> RequestTransport {
        RequestTransport {
            moduleId: module_id.clone(),
            payload: RequestData::new(mod_RequestData::OneOfdata::datachanges(data)),
            requestFunctionType: RequestFunctionType::DESTRUCTOR,
        }
    }

    pub fn create_UPDATE_DATA(module_id: &ModuleId, data: DataChanges) -> RequestTransport {
        RequestTransport {
            moduleId: module_id.clone(),
            payload: RequestData::new(mod_RequestData::OneOfdata::datachanges(data)),
            requestFunctionType: RequestFunctionType::UPDATE_DATA,
        }
    }

    pub fn create_PROCESS(module_id: &ModuleId, data: DataChanges) -> RequestTransport {
        RequestTransport {
            moduleId: module_id.clone(),
            payload: RequestData::new(mod_RequestData::OneOfdata::datachanges(data)),
            requestFunctionType: RequestFunctionType::PROCESS,
        }
    }
}
