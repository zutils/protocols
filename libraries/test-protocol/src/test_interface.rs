use test::Test;
use failure::{Error};
use protocols::pluginhandler::{MessageInfo, SubLibrary};

pub struct TestInterface;

impl SubLibrary for TestInterface {
    fn get_name(&self) -> String {
        return "Test".to_string();
    }

    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        let string: String = info.data.iter().map(|u: &u8| *u as char).collect();
        println!("Handling: {:?}", string);
        let structure: Test = serde_json::from_str(&string)?;
        println!("Received message: {:?}", structure);
        
        Ok(Vec::new())
    }

    fn get_schema_url() -> String {
        return include_str!("../schema_urls/test.txt").to_string();
    }
}

/// Non-standard dynamic function.
#[no_mangle]
pub extern fn generate_test_message(name: &str, data: &str) -> Result<String, Error> {
    // For now, just generate a default message
    let mut structure = Test::new();
    structure.set_name(name.to_string());
    structure.set_data(data.to_string());
    Ok(serde_json::to_string(&structure)?)
}
