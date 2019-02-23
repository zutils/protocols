#![allow(non_snake_case)]

use crate::{Destination, RpcData, Transport, VecModuleInfo, VecRpcData};
use crate::propagator::Propagator;
use crate::common::CommonModule;
use crate::transportresponse::{TransportResponse, TransportCombiner};

use crate::transport_autogen::transport::{DataType, RequestType, mod_DataType};

use failure::Error;

// Anywhere there is a common module, we want transports working with them.
impl<T> TransportToModuleGlue for T where T: CommonModule {}

/// These functions are the endpoints to the different modules.
pub trait TransportToModuleGlue: CommonModule {
    fn handle_transport(&self, transport: &Transport) -> Result<Vec<Transport>, Error> {
        // Pass on transport to proper function
        match transport.request_type {
            RequestType::GET_INFO => self.get_info_glue(transport),
            RequestType::RECEIVE_RPC_AS_CLIENT => self.receive_rpc_as_client_glue(transport),
            RequestType::RECEIVE_RPC_AS_SERVER => self.receive_rpc_as_server_glue(transport),
            RequestType::RECEIVE_PUBLIC_RPC => self.receive_public_rpc_glue(transport),
        }
    }

    fn get_info_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> {
        if let mod_DataType::OneOfresult::destination(msg) = &transport.payload.result {
            let module_ret = self.get_info(&msg)?;
            let result = mod_DataType::OneOfresult::vecmoduleinfo(module_ret);
            let ret = TransportResponse::create_Transport_result(result);
            Ok(vec![ret])    
        } else {
            Err(failure::format_err!("No Destination found!"))
        }
    }

    fn receive_rpc_as_client_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        if let mod_DataType::OneOfresult::rpcdata(msg) = &transport.payload.result {
            let module_ret = self.receive_rpc_as_client(&msg)?;
            let result = mod_DataType::OneOfresult::vecrpcdata(module_ret);
            let ret = TransportResponse::create_Transport_result(result);
            Ok(vec![ret])    
        } else {
            Err(failure::format_err!("No Destination found!"))
        }
    }

    fn receive_rpc_as_server_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
            if let mod_DataType::OneOfresult::rpcdata(msg) = &transport.payload.result {
                let module_ret = self.receive_rpc_as_server(&msg)?;
                let result = mod_DataType::OneOfresult::vecrpcdata(module_ret);
                let ret = TransportResponse::create_Transport_result(result);
                Ok(vec![ret])    
            } else {
                Err(failure::format_err!("No Destination found!"))
            }
    }

    fn receive_public_rpc_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        if let mod_DataType::OneOfresult::rpcdata(msg) = &transport.payload.result {
            let module_ret = self.receive_public_rpc(&msg)?;
            let result = mod_DataType::OneOfresult::vecrpcdata(module_ret);
            let ret = TransportResponse::create_Transport_result(result);
            Ok(vec![ret])    
        } else {
            Err(failure::format_err!("No Destination found!"))
        }
    }
}

/// This is glue to package up requests into Transports and unpacking them.
pub trait ModuleToTransportGlue: Propagator {
    fn get_info(&self, data: Destination) -> Result<VecModuleInfo, Error> {
        log::debug!("Propagating get_info({:?})", data);
        let transport = TransportRequest::create_GET_INFO(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecModuleInfo(transport_results)
    }

    fn receive_rpc_as_client(&self, data: RpcData) -> Result<VecRpcData, Error> {
        log::debug!("Propagating receive_rpc_as_client({:?})", data);
        let transport = TransportRequest::create_RECEIVE_RPC_AS_CLIENT(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecRpcData(transport_results)
    }

    fn receive_rpc_as_server(&self, data: RpcData) -> Result<VecRpcData, Error> {
        log::debug!("Propagating receive_rpc_as_server({:?})", data);
        let transport = TransportRequest::create_RECEIVE_RPC_AS_SERVER(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecRpcData(transport_results)
    }

    fn receive_public_rpc(&self, data: RpcData) -> Result<VecRpcData, Error> {
        log::debug!("Propagating receive_public_rpc({:?})", data);
        let transport = TransportRequest::create_RECEIVE_PUBLIC_RPC(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecRpcData(transport_results)
    }
}


pub struct TransportRequest;
impl TransportRequest {
    pub fn create_GET_INFO(data: Destination) -> Transport {
        Transport {
            destination: Some(data.schema.clone()),
            payload: DataType::new(mod_DataType::OneOfresult::destination(data)),
            request_type: RequestType::GET_INFO,
        }
    }

    pub fn create_RECEIVE_RPC_AS_CLIENT(data: RpcData) -> Transport {
        Transport {
            destination: Some(data.schema.clone()),
            payload: DataType::new(mod_DataType::OneOfresult::rpcdata(data)),
            request_type: RequestType::RECEIVE_RPC_AS_CLIENT,
        }
    }

    pub fn create_RECEIVE_RPC_AS_SERVER(data: RpcData) -> Transport {
        Transport {
            destination: Some(data.schema.clone()),
            payload: DataType::new(mod_DataType::OneOfresult::rpcdata(data)),
            request_type: RequestType::RECEIVE_RPC_AS_SERVER,
        }
    }

    pub fn create_RECEIVE_PUBLIC_RPC(data: RpcData) -> Transport {
        Transport {
            destination: Some(data.schema.clone()),
            payload: DataType::new(mod_DataType::OneOfresult::rpcdata(data)),
            request_type: RequestType::RECEIVE_PUBLIC_RPC,
        }
    }
}
