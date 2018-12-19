// Do not delete this file! At the moment, it is not autogenerated. It is manually generated.
#![allow(non_snake_case)]

use super::transport::{DataType, Destination, RequestType, RpcData, Transport, VecModuleInfo, 
    VecData, Data, VecRpcData, GenerateMessageInfo, DataType_oneof_result};

use crate::core::Propagator;
use crate::common::CommonModule;
use crate::transportresponse::{TransportResponse, TransportCombiner};

use failure::Error;

// Anywhere there is a common module, we want transports working with them.
impl<T> TransportToModuleGlue for T where T: CommonModule {}

/// These functions are the endpoints to the different modules.
pub trait TransportToModuleGlue: CommonModule {
    fn handle_transport(&self, transport: &Transport) -> Result<Vec<Transport>, Error> {
        // Pass on transport to proper function
        match transport.request_type {
            RequestType::GET_INFO => self.get_info_glue(transport),
            RequestType::GENERATE_MESSAGE => self.generate_message_glue(transport),
            RequestType::HANDLE_TRUSTED => self.handle_trusted_glue(transport),
            RequestType::RECEIVE_TRUSTED_RPC => self.receive_trusted_rpc_glue(transport),
            RequestType::RECEIVE_UNTRUSTED_RPC => self.receive_untrusted_rpc_glue(transport),
            RequestType::NONE => { println!("No request type to handle!"); Ok(Vec::new()) },
        }
    }

    fn get_info_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        let msg = transport.get_payload().get_destination();
        let module_ret = self.get_info(msg)?;
        let result = DataType_oneof_result::vecmoduleinfo(module_ret);
        let ret = TransportResponse::create_Transport_result(result);
        Ok(vec![ret])
    }

    fn generate_message_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        let msg = transport.get_payload().get_generatemessageinfo();
        let module_ret = self.generate_message(msg)?;
        let result = DataType_oneof_result::data(module_ret);
        let ret = TransportResponse::create_Transport_result(result);
        Ok(vec![ret])
    }

    fn handle_trusted_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        let msg = transport.get_payload().get_data();
        let module_ret = self.handle_trusted(msg)?;
        let result = DataType_oneof_result::vecdata(module_ret);
        let ret = TransportResponse::create_Transport_result(result);
        Ok(vec![ret])
    }

    fn receive_trusted_rpc_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        let msg = transport.get_payload().get_rpcdata();
        let module_ret = self.receive_trusted_rpc(msg)?;
        let result = DataType_oneof_result::vecrpcdata(module_ret);
        let ret = TransportResponse::create_Transport_result(result);
        Ok(vec![ret])
    }

    fn receive_untrusted_rpc_glue(&self, transport: &Transport) -> Result<Vec<Transport>, Error> { 
        let msg = transport.get_payload().get_rpcdata();
        let module_ret = self.receive_untrusted_rpc(msg)?;
        let result = DataType_oneof_result::vecrpcdata(module_ret);
        let ret = TransportResponse::create_Transport_result(result);
        Ok(vec![ret])
    }
}

/// This is glue to package up requests into Transports and unpacking them.
pub trait ModuleToTransportGlue: Propagator {
    fn get_info(&self, data: Destination) -> Result<VecModuleInfo, Error> {
        let transport = TransportRequest::create_GET_INFO(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecModuleInfo(transport_results)
    }

    fn generate_message(&self, data: GenerateMessageInfo) -> Result<Data, Error> { 
        let transport = TransportRequest::create_GENERATE_MESSAGE(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_Data(transport_results)
    }

    // handle of straight data is special because the data message contains the receiver.
    fn handle_trusted(&self, data: Data) -> Result<VecData, Error> { 
        let transport = TransportRequest::create_HANDLE_TRUSTED(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecData(transport_results)
    }

    fn receive_trusted_rpc(&self, data: RpcData) -> Result<VecRpcData, Error> {
        let transport = TransportRequest::create_RECEIVE_TRUSTED_RPC(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecRpcData(transport_results)
    }

    fn receive_untrusted_rpc(&self, data: RpcData) -> Result<VecRpcData, Error> {
        let transport = TransportRequest::create_RECEIVE_UNTRUSTED_RPC(data);
        let transport_results = self.propagate_transport(&transport);
        TransportCombiner::combine_to_VecRpcData(transport_results)
    }
}


pub struct TransportRequest;
impl TransportRequest {
    fn create_Transport_result(data: DataType_oneof_result) -> Transport {
        let mut data_type = DataType::new();
        data_type.result = Some(data);

        let mut ret = Transport::default();
        ret.set_payload(data_type);
        ret
    }

    pub fn create_GET_INFO(data: Destination) -> Transport {
        let destination = data.get_schema().clone();
        let result = DataType_oneof_result::destination(data);
        let mut transport = TransportRequest::create_Transport_result(result);
        transport.set_request_type(RequestType::GENERATE_MESSAGE);
        transport.set_destination(destination);
        transport
    }

    pub fn create_GENERATE_MESSAGE(data: GenerateMessageInfo) -> Transport {
        let destination = data.get_schema().clone();
        let result = DataType_oneof_result::generatemessageinfo(data);
        let mut transport = TransportRequest::create_Transport_result(result);
        transport.set_request_type(RequestType::GENERATE_MESSAGE);
        transport.set_destination(destination);
        transport
    }

    pub fn create_HANDLE_TRUSTED(data: Data) -> Transport {
        let destination = data.get_schema().clone();
        let result = DataType_oneof_result::data(data);
        let mut transport = TransportRequest::create_Transport_result(result);
        transport.set_request_type(RequestType::HANDLE_TRUSTED);
        transport.set_destination(destination);
        transport
    }

    pub fn create_RECEIVE_TRUSTED_RPC(data: RpcData) -> Transport {
        let destination = data.get_schema().clone();
        let result = DataType_oneof_result::rpcdata(data);
        let mut transport = TransportRequest::create_Transport_result(result);
        transport.set_request_type(RequestType::RECEIVE_TRUSTED_RPC);
        transport.set_destination(destination);
        transport
    }

    pub fn create_RECEIVE_UNTRUSTED_RPC(data: RpcData) -> Transport {
        let destination = data.get_schema().clone();
        let result = DataType_oneof_result::rpcdata(data);
        let mut transport = TransportRequest::create_Transport_result(result);
        transport.set_request_type(RequestType::RECEIVE_UNTRUSTED_RPC);
        transport.set_destination(destination);
        transport
    }
}
