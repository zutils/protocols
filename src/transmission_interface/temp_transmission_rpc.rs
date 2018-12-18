use super::transmission::Error as TError;
//use super::transmission::{mod_DataType};
use super::transmission::DataType_oneof_result;
use super::transmission::{Schema, Empty, DataType, RequestType, RpcData, Transmission, VecModuleInfo, VecData, Data, VecRpcData, GenerateMessageInfo};
use crate::transport::Propagator;

use failure::Error;

pub trait CommonModule {
    fn get_info(&self, p: Empty) -> Result<VecModuleInfo, Error>;

    fn generate_default_message(&self, p: GenerateMessageInfo) -> Result<Data, Error>;

    fn handle_trusted(&self, p: Data) -> Result<VecData, Error>;

    fn receive_trusted_rpc(&self, p: RpcData) -> Result<VecRpcData, Error>;

    fn receive_untrusted_rpc(&self, p: RpcData) -> Result<VecRpcData, Error>;
}

// Anywhere there is a common module, we want transmissions working with them.
impl<T> TransportationToModuleGlue for T where T: CommonModule {}

/// These functions are the endpoints to the different modules.
pub trait TransportationToModuleGlue: CommonModule {
    fn handle_transmission(&self, transmission: &Transmission) -> Vec<Transmission> {
        // Pass on transmission to proper function
        match transmission.request_type {
            RequestType::GET_INFO => self.get_info_glue(transmission),
            RequestType::GENERATE_DEFAULT_MESSAGE => self.generate_default_message_glue(transmission),
            RequestType::HANDLE_TRUSTED => self.handle_trusted_glue(transmission),
            RequestType::RECEIVE_TRUSTED_RPC => self.receive_trusted_rpc_glue(transmission),
            RequestType::RECEIVE_UNTRUSTED_RPC => self.receive_untrusted_rpc_glue(transmission),
            RequestType::NONE => { println!("No request type to handle!"); Vec::new() },
        }
    }

    fn get_info_glue(&self, _transmission: &Transmission) -> Vec<Transmission> { 
        // get_info doesn't need to use transmission as there is no input. It goes to everyone!
        let module_ret = self.get_info(Empty::new());

        let data_type_result = match module_ret {
            Ok(vecmoduleinfo) => DataType_oneof_result::vecmoduleinfo(vecmoduleinfo),
            Err(e) => DataType_oneof_result::error( {
                let mut err = TError::new(); 
                err.set_error(format!("{:?}",e));
                err
            }),
        };

        let mut data_type = DataType::new();
        data_type.result = Some(data_type_result);

        let mut ret = Transmission::default();
        ret.set_payload(data_type);
        vec![ret]
    }

    fn generate_default_message_glue(&self, _transmission: &Transmission) -> Vec<Transmission> { 
        println!("generate_default_message(transmission) not yet implemented!"); 
        Vec::new()
    }
    fn handle_trusted_glue(&self, _transmission: &Transmission) -> Vec<Transmission> { 
        println!("handle_trusted(transmission) not yet implemented!"); 
        Vec::new()
    }
    fn receive_trusted_rpc_glue(&self, _transmission: &Transmission) -> Vec<Transmission> { 
        println!("receive_trusted_rpc(transmission) not yet implemented!"); 
        Vec::new()
    }
    fn receive_untrusted_rpc_glue(&self, _transmission: &Transmission) -> Vec<Transmission> { 
        println!("receive_untrusted_rpc(transmission) not yet implemented!"); 
        Vec::new()
    }
}

/// This is glue to package up requests into Transmissions and unpacking them.
pub trait ModuleToTransportationGlue: Propagator {
    fn get_info(&self, _empty: Empty) -> VecModuleInfo {
        // Generate transmission
        let mut transmission = Transmission::default();
        transmission.set_request_type(RequestType::GET_INFO);
        //transmission.set_destination(...) // No destination means that we return a vec
        //transmission.set_payload(...) // No payload for get_info
            
        // Propagate
        let transportation_results = self.propagate_transmission(&transmission);

        // Combine results (handle every error for some reason)
        let mut result = VecModuleInfo::default();
        for mut transportation in transportation_results {
            if transportation.has_payload() && transportation.get_payload().result.is_some() {
                let payload = transportation.take_payload();
                match payload.result.unwrap() { // We can unwrap because we just checked is_some() above.
                    DataType_oneof_result::error(ref m) => println!("Module returned error: {:?}", m),
                    DataType_oneof_result::data(ref _m) => println!("Error! Payload is incorrect type: data"),
                    DataType_oneof_result::vecmoduleinfo(ref m) => {
                        for new in m.vec.clone().into_iter() { // No append function exists for RepeatedField.
                            result.vec.push(new);
                        }
                    },
                    DataType_oneof_result::rpcdata(ref _m) => println!("Error! Payload is incorrect type: rpcdata"),
                    DataType_oneof_result::generatemessageinfo(ref _m) => println!("Error! Payload is incorrect type: generatemessageinfo"),
                    DataType_oneof_result::empty(ref _m) => println!("Error! Payload is incorrect type: empty"),
                    DataType_oneof_result::vecdata(ref _m) => println!("Error! Payload is incorrect type: vecdata"),
                    DataType_oneof_result::vecdatarpc(ref _m) => println!("Error! Payload is incorrect type: vecdatarpc"),
                }
            } else {
                println!("No payload exists!");
            }
        }

        result
    }

    fn generate_default_message(&self, _destination: Schema, _message_info: GenerateMessageInfo) -> Result<Data, Error> { 
        Err(failure::format_err!("generate_default_message unimplemented!"))
    }

    // handle of straight data is special because the data message contains the receiver.
    fn handle_trusted(&self, _data: Data) -> Result<VecData, Error> { 
        Err(failure::format_err!("handle_trusted unimplemented!"))
    }

    fn receive_trusted_rpc(&self, _destination: Schema, _rpc: RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("receive_trusted_rpc unimplemented!")) 
    }

    fn receive_untrusted_rpc(&self, _destination: Schema, _rpc: RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("receive_untrusted_rpc unimplemented!"))
    }
}

