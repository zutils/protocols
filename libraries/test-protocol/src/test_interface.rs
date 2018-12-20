use crate::test_autogen::test;
use failure::Error;
use protocols::{CommonModule, Data, ModuleInfo, VecModuleInfo, VecData, Destination, GenerateMessageInfo, VecRpcData, RpcData};
use protocols::utils::{ToDataConverter, FromDataConverter, schema_ipfs_from_str};

static SCHEMA_URL: &str = include_str!("../schema_urls/test.txt");

pub struct TestInterface;

impl ToDataConverter for test::Test {}

impl CommonModule for TestInterface {
    fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, Error> {
        let mut info = ModuleInfo::new();
        info.set_name("Test".to_string());
        info.set_schema(schema_ipfs_from_str(SCHEMA_URL)); // Future: Perhaps return multiple modules and append "/Test" to SCHEMA_URL
        
        let mut ret = VecModuleInfo::new();
        ret.vec = protobuf::RepeatedField::from_vec(vec![info]);
        Ok(ret)
    }

    fn generate_message(&self, data: &GenerateMessageInfo) -> Result<Data, Error> {
        use std::str;
        let template = data.get_template();
        let args = data.get_args();
        match template {
            "Test" => {
                let name = str::from_utf8(&(args[0]))?;
                let data = str::from_utf8(&(args[1]))?;
                let msg = generate_test(name, data)?;
                Ok(msg.to_Data(&schema_ipfs_from_str(SCHEMA_URL))?)
            },
            _ => Err(failure::format_err!("Unrecognized template {:?}. 'Root' available.", template)),
        }
    }

    fn handle_trusted(&self, data: &Data) -> Result<VecData, Error> {
        let (schema, test) = data.unwrap::<test::Test>()?;
        println!("Received Test Message: ({:?},{:?})", schema, test);

        let ret = VecData::new();
        Ok(ret)
    }

    fn receive_trusted_rpc(&self, _data: &RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No Trusted Rpc for {:?}", SCHEMA_URL))
    }

    fn receive_untrusted_rpc(&self, _data: &RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No Untrusted Rpc for {:?}", SCHEMA_URL))
    }
}

fn generate_test(name: &str, data: &str) -> Result<test::Test, Error> {   
    let mut structure = test::Test::new();
    structure.set_name(name.to_string());
    structure.set_data(data.to_string());
    Ok(structure)
}

