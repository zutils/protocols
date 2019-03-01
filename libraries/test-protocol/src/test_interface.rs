use crate::test_autogen::test;
use protocols::{CommonModule, ModuleInfo, VecModuleInfo, Destination, VecRpcData, RpcData};

static SCHEMA_URL: &str = include_str!("../schema_urls/test.txt");
struct ClientRPCHandler;
struct ServerRPCHandler;
struct PublicRPCHandler;
//impl test::ClientRPC for ClientRPCHandler {}
impl test::ServerRPC for ServerRPCHandler {}
impl test::PublicRPC for PublicRPCHandler {}

pub struct Interface;
impl CommonModule for Interface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, failure::Error> {
        let info = ModuleInfo::new(SCHEMA_URL.into(), "test".to_string());
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
        log::info!("Recieved test data {:?}", data);
        Ok(vec![])
    }
}
