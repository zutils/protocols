use crate::autogen_protobuf::test;

use protocols::{CommonModelFunctions, ModuleInfo, VecModuleInfo, Destination, VecRpcData, RpcData};

struct ClientRPCHandler;
struct ServerRPCHandler;
struct PublicRPCHandler;
//impl test::ClientRPC for ClientRPCHandler {}
impl test::ServerRPC for ServerRPCHandler {}
impl test::PublicRPC for PublicRPCHandler {}

pub struct Interface;
impl CommonModelFunctions for Interface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, failure::Error> {
        let info = ModuleInfo::new(test::SCHEMA_URL.into(), "test".to_string());
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
//protocols::implCommonModelFunctions!(test, SCHEMA_URL);

impl test::ClientRPC for ClientRPCHandler {
    fn send_test(&self, data: test::Test) -> Result<Vec<RpcData>, failure::Error> {
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
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::path::PathBuf;

    #[test]
    fn test_send_receive_wasm() {
        if let Err(e) = test_send_receive(&PathBuf::from("./target/wasm32-unknown-wasi/release/test-protocol.wasm")) {
            panic!("{:?}", e);
        }
        println!("test_send_receive_wasm test passed!");
    }

    /*#[test]
    fn test_send_receive_dll() -> Result<(), failure::Error> {
        test_send_receive(&PathBuf::from("./target/debug/deps/test-protocol.dll"))?;
        Ok(())
    }*/

    fn test_send_receive(plugin_filename: &PathBuf) -> Result<(), failure::Error> {
        use crate::autogen_protobuf::test::ClientRPC; // For send_test()
        protocols::logging::initialize_standard_logging("")?;

        // We need Sync and Send memory for send_test(...)
        let data = Arc::new(Mutex::new(protocols::RpcData::default()));
        let data_clone = data.clone();

        // In our case, we do not transmit, we just write to some data.
        let transmit_rpc_function = move |rpc| { *data_clone.lock().unwrap() = rpc; Ok(())};
        let client = test::SendClientRPC { func: Box::new(transmit_rpc_function) };
        let test_data = test::Test::new("Test Name".to_string(), "Test Data".to_string());
        client.send_test(test_data)?; 

        let data = data.lock().unwrap().clone();
        let bytes = quick_protobuf::serialize_into_vec(&data)?;
        receive_plugin(plugin_filename, &bytes)?;
        Ok(())
    }

    fn receive_plugin(plugin_filename: &PathBuf, data: &[u8]) -> Result<(), failure::Error> {
        //use protocols::{ModelFunctionToTransportGlue};

        // Initialize plugin handler. The PluginHandler is ALSO our module root.
        let mut handler = protocols::PluginHandler::new();
        handler.load_and_cache_plugin(plugin_filename)?;
        
        // Convert received bytes to a Data type.
        let data: protocols::RpcData = quick_protobuf::deserialize_from_slice(&data)?; 

        // Propogate through the handler tree to find a module matching the schema.
        // Note that we do not pass in a schema for data as the data structure already contains the schema it is supposed to be used for.
        handler.receive_rpc_as_client(data)?;

        Ok(())
    }
}
