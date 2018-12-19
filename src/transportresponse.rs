//! Payload functions offer serializers and deserializers for common Transport payloads
#![allow(non_snake_case)]

use crate::transport_autogen::transport::Error as TError;
use crate::transport_autogen::transport::{Transport, DataType_oneof_result, Data, DataType, RpcData,
    ModuleInfo, VecModuleInfo, VecData, VecRpcData};

use failure::Error;

pub struct TransportResponse;
impl TransportResponse {
    pub fn create_Transport_result(data: DataType_oneof_result) -> Transport {
        let mut data_type = DataType::new();
        data_type.result = Some(data);

        let mut ret = Transport::default();
        ret.set_payload(data_type);
        ret
    }

    pub fn create_Error(e: &str) -> Transport {
        let mut err = TError::new(); 
        err.set_error(format!("{:?}",e));
        TransportResponse::create_Transport_result(DataType_oneof_result::error(err))
    }
}

/// Returning transports can be poisoned via webasm unsafe code and can be a security risk if not handled properly!!!
pub struct TransportCombiner;
impl TransportCombiner {
    fn filter_out_and_print_errors(results: Vec<Transport>) -> Vec<DataType> {
        let (errors, results): (Vec<_>,Vec<_>) = results.into_iter()
            .map(|mut transport| transport.take_payload() )
            .partition(|payload| payload.has_error() );

        let _unused: Vec<_> = errors.into_iter()
            .map(|mut payload| payload.take_error().error )
            .inspect(|e| println!("Error from transport! {:?}", e))
            .collect();

        results            
    }

    pub fn combine_to_VecModuleInfo(results: Vec<Transport>) -> Result<VecModuleInfo, Error> {
        let results = TransportCombiner::filter_out_and_print_errors(results);

        let infos: Vec<ModuleInfo> = results.into_iter()
            .filter(|payload| payload.has_vecmoduleinfo() )
            .map(|mut payload| payload.take_vecmoduleinfo() )
            .map(|structure| structure.vec.into_vec() )
            .flatten().collect();
        
        let mut ret = VecModuleInfo::default();
        ret.vec = protobuf::RepeatedField::from_vec(infos);
        Ok(ret)
    }

    pub fn combine_to_Data(results: Vec<Transport>) -> Result<Data, Error> {
        let results = TransportCombiner::filter_out_and_print_errors(results);

        let mut infos: Vec<Data> = results.into_iter()
            .filter(|payload| payload.has_data() )
            .map(|mut payload| payload.take_data() )
            .collect();

        if infos.len() > 1 {
            println!("combine_to_Data(...) has more than one result! Returning first one.")
        } 

        let item = infos.pop().ok_or(failure::format_err!("No response for data request!"))?;
        Ok(item)
    }

    pub fn combine_to_VecData(results: Vec<Transport>) -> Result<VecData, Error> {
        let results = TransportCombiner::filter_out_and_print_errors(results);

        let infos: Vec<Data> = results.into_iter()
            .filter(|payload| payload.has_vecdata() )
            .map(|mut payload| payload.take_vecdata() )
            .map(|structure| structure.vec.into_vec() )
            .flatten().collect();
        
        let mut ret = VecData::default();
        ret.vec = protobuf::RepeatedField::from_vec(infos);
        Ok(ret)
    }

    pub fn combine_to_VecRpcData(results: Vec<Transport>) -> Result<VecRpcData, Error> {
        let results = TransportCombiner::filter_out_and_print_errors(results);

        let infos: Vec<RpcData> = results.into_iter()
            .filter(|payload| payload.has_vecrpcdata() )
            .map(|mut payload| payload.take_vecrpcdata() )
            .map(|structure| structure.vec.into_vec() )
            .flatten().collect();
        
        let mut ret = VecRpcData::default();
        ret.vec = protobuf::RepeatedField::from_vec(infos);
        Ok(ret)
    }
}
