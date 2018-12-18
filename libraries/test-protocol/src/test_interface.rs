use test::Test;
use failure::Error;
use protocols::transmission::{Schema, Data, VecModuleInfo, VecData};
use protocols::temp_transmission_rpc::{CommonModule, TransportationToModuleGlue};

static SCHEMA_URL: &str = include_str!("../schema_urls/test.txt");

#[derive(TransportationToModuleGlue)]
pub struct TestInterface;

impl ToDataConverter for Test {}

impl CommonModule for TestInterface {
    fn get_info(&self, _: Empty) -> Result<VecModuleInfo, Error> {
        let info = VecModuleInfo::new();
        info.set_name("Test".to_string());
        info.set_schema(SCHEMA_URL.to_string()); // Future: Perhaps return multiple modules and append "/Test" to SCHEMA_URL
        vec![info]
    }

    fn generate_default_message(&self, data: GenerateMessageInfo) -> Result<Data, Error> {
        use std::str;
        let template = data.get_template();
        let args = data.get_args();
        match template {
            "Test" => {
                let name: str::from_utf8f(&args[0]);
                let data = str::from_utf8(&args[1]);
                let msg = generate_test(name, data);
                Ok(msg.to_data(SCHEMA_URL)?)
            },
            _ => Err(failure::format_err!("Unrecognized template {:?}. 'Root' available.", template)),
        }
    }

    fn handle_trusted(&self, data: Data) -> Result<VecData, Error> {
        let (schema, test) = from_data::<Test>(&data)?;
        println!("Received Test Message: ({:?},{:?})", schema, test);
        Ok(Vec::new())
    }

    fn receive_trusted_rpc(&self, data: RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No Trusted Rpc for {}", SCHEMA_URL));
    }

    fn receive_untrusted_rpc(&self, data: RpcData) -> Result<VecRpcData, Error> {
        Err(failure::format_err!("No Untrusted Rpc for {}", SCHEMA_URL));
    }
}

fn generate_test(name: &Schema, data: &[u8]) -> Result<Test, Error> {   
    let mut structure = Test::new();
    structure.set_name(name.to_string());
    structure.set_data(data.to_string());
    structure
}

