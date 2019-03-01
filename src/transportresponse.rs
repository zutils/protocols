//! Payload functions offer serializers and deserializers for common Transport payloads
#![allow(non_snake_case)]

use crate::transport_autogen::transport::Error as TError;
use crate::{Transport, Data, RpcData, ModuleInfo, VecModuleInfo, VecData, VecRpcData};
use crate::transport_autogen::transport::{mod_DataType, DataType};

use failure::Error;

pub struct TransportResponse;
impl TransportResponse {
    pub fn create_Transport_result(data: mod_DataType::OneOfresult) -> Transport {       
        Transport {
            payload: DataType::new(data),
            ..Default::default()
        }
    }

    pub fn create_Error(e: &str) -> Transport {
        let err = TError::new(e.to_string()); 
        TransportResponse::create_Transport_result(mod_DataType::OneOfresult::error(err))
    }
}

/// Returning transports can be poisoned via webasm unsafe code and can be a security risk if not handled properly!!!
pub struct TransportCombiner;
impl TransportCombiner {
    fn filter_some_and_print_errors<'a>(results: Vec<Transport>) -> Vec<mod_DataType::OneOfresult> {
        results.into_iter()
            .filter_map(|transport| match transport.payload.result {
                mod_DataType::OneOfresult::error(e) => { log::debug!("{:?}", e.val); None },
                res => Some(res),
            }).collect()       
    }

    pub fn combine_to_VecModuleInfo(results: Vec<Transport>) -> Result<VecModuleInfo, Error> {
        let results = TransportCombiner::filter_some_and_print_errors(results);

        let infos: Vec<ModuleInfo> = results.into_iter()
            .filter_map(|result| match result { 
                mod_DataType::OneOfresult::vecmoduleinfo(p) => Some(p.vec), 
                _ => None, 
            })
            .flatten().collect();
        
        Ok(VecModuleInfo::new(infos))
    }

    pub fn combine_to_Data(results: Vec<Transport>) -> Result<Data, Error> {
        let results = TransportCombiner::filter_some_and_print_errors(results);

        let mut infos: Vec<Data> = results.into_iter()
            .filter_map(|result| match result { 
                mod_DataType::OneOfresult::data(p) => Some(p), 
                _ => None, 
            })
            .collect();

        if infos.len() > 1 {
            log::debug!("combine_to_Data(...) has more than one result! Returning first one.")
        } 

        Ok(infos.pop().ok_or(failure::format_err!("No response for data request!"))?)
    }

    pub fn combine_to_VecData(results: Vec<Transport>) -> Result<VecData, Error> {
        let results = TransportCombiner::filter_some_and_print_errors(results);

        let infos: Vec<Data> = results.into_iter()
            .filter_map(|result| match result { 
                mod_DataType::OneOfresult::vecdata(p) => Some(p.vec), 
                _ => None, 
            })
            .flatten().collect();
        
        Ok(VecData::new(infos))
    }

    pub fn combine_to_VecRpcData(results: Vec<Transport>) -> Result<VecRpcData, Error> {
        let results = TransportCombiner::filter_some_and_print_errors(results);

        let infos: Vec<RpcData> = results.into_iter()
            .filter_map(|result| match result { 
                mod_DataType::OneOfresult::vecrpcdata(p) => Some(p.vec), 
                _ => None, 
            })
            .flatten().collect();
        
        Ok(VecRpcData::new(infos))
    }
}

impl From<Vec<RpcData>> for VecRpcData {
    fn from(f: Vec<RpcData>) -> Self {
        VecRpcData::new(f)
    }
}