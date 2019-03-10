use crate::autogen::test;
use crate::autogen::schema_map;

use protocols::{CommonModule, ModuleInfo, VecModuleInfo, Destination, VecRpcData, RpcData};

struct ClientRPCHandler;
struct ServerRPCHandler;
struct PublicRPCHandler;
//impl test::ClientRPC for ClientRPCHandler {}
impl test::ServerRPC for ServerRPCHandler {}
impl test::PublicRPC for PublicRPCHandler {}

pub struct Interface;
impl CommonModule for Interface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, failure::Error> {
        let info = ModuleInfo::new(schema_map::SCHEMA_URL_TEST.into(), "test".to_string());
        Ok(VecModuleInfo::new(vec![info]))
    }

    fn receive_rpc_as_client(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
        test::handle_ClientRPC(data, ClientRPCHandler{})
    }

    fn receive_rpc_as_server(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
        test::handle_ServerRPC(data, ServerRPCHandler{})
    }

    fn receive_public_rpc(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
        test::handle_PublicRPC(data, PublicRPCHandler{})
    }
}
//protocols::implCommonModule!(test, SCHEMA_URL);

impl test::ClientRPC for ClientRPCHandler {
    fn receive_test(&self, data: test::Test) -> Result<Vec<RpcData>, failure::Error> {
        log::info!("Recieved test data {:?}.", data);
        log::info!(r#"Testing to see if name is "Test Name" and data is "Test Data"#);
        if data.name != "Test Name" || data.data != "Test Data" {
            panic!("Test Unsuccessful!");
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_send_receive() {
        if let Err(e) = protocols::logging::initialize_standard_logging("") {
            panic!("Error: {:?}", e);
        }

        let rpc_send_data = get_send_data();
        if let Err(e) = rpc_send_data {
            panic!("Error: {:?}", e);
        }
        if let Err(e) = receive(&rpc_send_data.unwrap()) {
            panic!("Error: {:?}", e);
        }
    }

    fn get_send_data() -> Result<Vec<u8>, failure::Error> {
        use super::*;

        let test_data = test::Test::new("Test Name".to_string(), "Test Data".to_string());
        
        let rpc = protocols::RpcData {
            method_name: "ClientRPC/receive_test".to_string(),
            schema: schema_map::SCHEMA_URL_TEST.into(),
            serialized_rpc_arg: quick_protobuf::serialize_into_vec(&test_data)?,
            ..Default::default()
        };

        log::info!("Serializing RpcData: {:?}", rpc);

        let send_data = quick_protobuf::serialize_into_vec(&rpc)?;
        Ok(send_data)
    }

    fn receive(data: &[u8]) -> Result<(), failure::Error> {
        use std::path::PathBuf;
        use protocols::{ModuleToTransportGlue, DynamicLibraryLoader};

        // Initialize plugin handler. The PluginHandler is ALSO our module root.
        let handler = protocols::PluginHandler::new();
        handler.load_plugin(&PathBuf::from("./target/debug/deps/test_protocol.dll"))?;
        
        // Convert received bytes to a Data type.
        let data: protocols::RpcData = quick_protobuf::deserialize_from_slice(&data)?; 

        // Propogate through the handler tree to find a module matching the schema.
        // Note that we do not pass in a schema for data as the data structure already contains the schema it is supposed to be used for.
        handler.receive_rpc_as_client(data)?;

        Ok(())
    }
}
