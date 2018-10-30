#[macro_use] extern crate serde_derive;
extern crate protobuf;
extern crate serde_json;
extern crate serde;
extern crate failure;
extern crate protocols;

pub mod test;
use test::Test;
use failure::Error;
use protocols::pluginmanager::PluginManager;

#[no_mangle]
pub extern fn get_name() -> String {
    return "Test".to_string();
}

#[no_mangle]
pub extern fn handle(manager: &PluginManager, data: &[u8]) -> Result<(), Error> {
    let string: String = data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {:?}", data);
    let structure: Test = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);
    
    Ok(Vec::new())
}

#[no_mangle]
pub extern fn generate_message(template_name: &str) -> Result<String, Error> {
    // For now, just generate a default message
    let structure = Test::new();
    Ok(serde_json::to_string(&structure)?)
}

#[no_mangle]
pub extern fn get_schema_url() -> String{
    return include_str!("../schema_url.txt").to_string();
}
