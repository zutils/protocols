//! Payload functions offer serializers and deserializers for common Transport payloads
#![allow(non_snake_case)]

use crate::autogen_protobuf::transport::*;
use failure::Error;
use std::convert::TryInto;

pub struct TransportResponse;
impl TransportResponse {
    pub fn create(data: mod_ReturnData::OneOfdata) -> ReturnTransport {       
        ReturnTransport::new(vec![ReturnData::new(data)])
    }

    pub fn create_TransportError(e: &str) -> ReturnTransport {
        let err = TransportError::new(e.to_string()); 
        TransportResponse::create(err.into())
    }
}

// Returning transports can be poisoned via wasm unsafe code and can be a security risk if not handled properly!!!
fn filter_some_and_print_errors(ret: ReturnTransport) -> Vec<mod_ReturnData::OneOfdata> {
    ret.vec.into_iter()
        .filter_map(|data| match data.data {
            mod_ReturnData::OneOfdata::error(e) => { log::debug!("{:?}", e.val); None },
            res => Some(res),
        }).collect()       
}

impl TryInto<VecId> for ReturnTransport {
    type Error = Error;
    fn try_into(self) -> Result<VecId, Self::Error> {
        let results = filter_some_and_print_errors(self);

        let data: Vec<Id> = results.into_iter()
            .filter_map(|result| match result { 
                mod_ReturnData::OneOfdata::submodelids(p) => Some(p.vec), 
                _ => None, 
            })
            .flatten().collect();
        
        Ok(VecId::new(data))
    }
}


impl TryInto<VecDataChanges> for ReturnTransport {
    type Error = Error;
    fn try_into(self) -> Result<VecDataChanges, Self::Error> {
        let results = filter_some_and_print_errors(self);

        let data: Vec<DataChanges> = results.into_iter()
            .filter_map(|result| match result { 
                mod_ReturnData::OneOfdata::VecDataChanges(p) => Some(p.vec), 
                _ => None, 
            })
            .flatten().collect();
        
        Ok(VecDataChanges::new(data))
    }
}

impl TryInto<DataChanges> for ReturnTransport {
    type Error = Error;
    fn try_into(self) -> Result<DataChanges, Self::Error> {
        let results = filter_some_and_print_errors(self);

        let data: Vec<DataChanges> = results.into_iter()
            .filter_map(|result| match result { 
                mod_ReturnData::OneOfdata::datachanges(p) => Some(p), 
                _ => None, 
            })
            .collect();

        if data.len() > 1 {
            return Err(failure::format_err!("Warning! Multiple responses. Expecting only one!"));
        }

        if data.len() == 0 {
            return Err(failure::format_err!("Warning! Expecting DataChanges!"));
        }
        
        Ok(data.first().unwrap().clone()) // We can unwrap because we already checked the length above.
    }
}
