use crate::test_autogen::test;
use failure::Error;
use protocols::{CommonModule, ModuleInfo, VecModuleInfo, Destination, VecRpcData, RpcData};

static SCHEMA_URL: &str = include_str!("../schema_urls/test.txt");

pub struct TestInterface;

impl CommonModule for TestInterface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, Error> {
        let info = ModuleInfo::new(Some(SCHEMA_URL.into()), Some("Test".into()));
        Ok(VecModuleInfo::new(vec![info]))
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
        let serialized_arg = match &data.serialized_rpc_arg {
            Some(arg) => quick_protobuf::deserialize_from_slice(&arg)?,
            None => return Err(failure::format_err!("No Arg!")),
        };

        let additional_rpcs = match data.method_name {
            Some(ref s) if s == "ClientRPC/receive_test" => {
                let _empty = ClientRPCHandler::receive_test(serialized_arg);
                Vec::new()
            },
            _ => Vec::new(),
        };

        Ok(VecRpcData::new(additional_rpcs))
    }

    fn receive_test(data: test::Test) -> test::Empty {
        log::info!("Recieved test data {:?}", data);
        test::Empty::new()
    }
}
