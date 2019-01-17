use crate::test_autogen::test;
use failure::Error;
use protocols::{CommonModule, Data, ModuleInfo, VecModuleInfo, Destination, VecRpcData, RpcData};
use protocols::utils::{schema_ipfs_from_str};

static SCHEMA_URL: &str = include_str!("../schema_urls/test.txt");

pub struct TestInterface;

impl CommonModule for TestInterface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, Error> {
        let mut info = ModuleInfo::new();
        info.set_name("Test".to_string());
        info.set_schema(schema_ipfs_from_str(SCHEMA_URL));
        
        let mut ret = VecModuleInfo::new();
        ret.vec = protobuf::RepeatedField::from_vec(vec![info]);
        Ok(ret)
    }

    fn receive_rpc_as_client(&self, data: &RpcData) -> Result<VecRpcData, Error> {
        ClientRPCHandler::handle(data)
    }

    fn receive_rpc_as_server(&self, _data: &RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No Untrusted Rpc for {:?}", SCHEMA_URL))
    }

    fn receive_public_rpc(&self, _data: &RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No public Rpc for {:?}", SCHEMA_URL))
    }
}

struct ClientRPCHandler;
impl ClientRPCHandler {
    fn handle(data: &RpcData) -> Result<VecRpcData, Error> {
        let additional_rpcs = match data.method_name.as_str() {
            "ClientRPC/receive_test" => {
                let arg = protobuf::parse_from_bytes(data.get_serialized_rpc_arg())?;
                let _empty = ClientRPCHandler::receive_test(arg);
                Vec::new()
            },
            _ => Vec::new(),
        };

        let mut ret = VecRpcData::new();
        ret.vec = protobuf::RepeatedField::from_vec(additional_rpcs);
        Ok(ret)
    }

    fn receive_test(data: Data) -> test::Empty {
        log::info!("Recieved test data {:?}", data);
        test::Empty::new()
    }
}
