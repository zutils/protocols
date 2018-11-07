#[macro_use] extern crate serde_derive;
extern crate protobuf;
extern crate serde_json;
extern crate serde;
extern crate failure;
extern crate protocols;

pub mod test;
use test::Test;
use failure::Error;
use protocols::pluginhandler::MessageInfo;

#[no_mangle]
pub extern fn get_name() -> String {
    return "Test".to_string();
}

#[no_mangle]
pub extern fn handle(info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
    let string: String = info.data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {:?}", string);
    let structure: Test = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);
    
    Ok(Vec::new())
}

#[no_mangle]
pub extern fn get_schema_url() -> String{
    return include_str!("../schema_url.txt").to_string();
}

// This should be replaced with a way to query the RPC.
#[no_mangle]
pub extern fn get_nonstandard_library_interface_functions() -> Vec<&'static str> {
    vec!["generate_test_message(name: &str, data: &str) -> Result<String, Error>"]
}

///////////Non-Standard////////////
#[no_mangle]
pub extern fn generate_test_message(name: &str, data: &str) -> Result<String, Error> {
    // For now, just generate a default message
    let mut structure = Test::new();
    structure.set_name(name.to_string());
    structure.set_data(data.to_string());
    Ok(serde_json::to_string(&structure)?)
}
