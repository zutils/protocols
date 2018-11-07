#[macro_use] extern crate serde_derive;
extern crate protobuf;
extern crate serde_json;
extern crate serde;
extern crate failure;
extern crate protocols;

pub mod rootmessage;
use rootmessage::RootMessage;
use failure::{Error};
use protocols::pluginhandler::MessageInfo;

#[no_mangle]
pub extern fn get_name() -> String {
    return "Rootmessage".to_string();
}

#[no_mangle]
pub extern fn handle(info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
    let string: String = info.data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {}", string);
    let structure: RootMessage = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);

    Ok(vec![create_submessage(structure, info)])
}

#[no_mangle]
pub extern fn get_schema_url() -> String{
    return include_str!("../schema_url.txt").to_string();
}

// This should be replaced with a way to query the RPC.
#[no_mangle]
pub extern fn get_nonstandard_library_interface_functions() -> Vec<&'static str> {
    vec!["generate_root_message(schema_url: &str, data: &[u8]) -> Result<String, Error>"]
}

/// Non-standard function.
#[no_mangle]
pub extern fn generate_root_message(schema_url: &str, data: &[u8]) -> Result<String, Error> {   
    let mut structure = RootMessage::new();
    structure.set_schema_url(schema_url.to_string());
    structure.set_unencrypted_message(data.to_vec());
    Ok(serde_json::to_string(&structure)?)
}

fn create_submessage(root: RootMessage, info: MessageInfo) -> MessageInfo {
    let schema_url = root.get_schema_url();
    let unencrypted_message = root.get_unencrypted_message();

    // Create history with 1 more element
    let mut history = info.history;
    history.push(schema_url.to_string());

    MessageInfo::new(history, schema_url, unencrypted_message)
}