extern crate protobuf;
extern crate serde_json;
extern crate serde;
extern crate serde_derive;
extern crate failure;
extern crate protocols;

pub mod rootmessage;
use rootmessage::RootMessage;
use failure::Error;
use protocols::pluginmanager::{PluginManager};

#[no_mangle]
pub extern fn get_name() -> String {
    return "Rootmessage".to_string();
}

#[no_mangle]
pub extern fn handle(manager: &PluginManager, data: &[u8]) -> Result<(), Error> {
    let string: String = data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {}", string);
    let structure: RootMessage = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);

    manager.handle_msg_and_submsgs(structure.get_schema_url(), structure.get_unencrypted_message());
    handle_submessages(manager, structure.get_unencrypted_message());
}

// This may represent a problem if root-message recurses itself.
fn handle_submessages(manager: &PluginManager, schema_url: &str, data: &[u8]) -> Result<(), Error> {
    manager.handle_msg_and_submsgs(schema_url, data);
}

#[no_mangle]
pub extern fn generate_message(_template_name: &str) -> Result<String, Error> {
    // for now, just generate a default message.
    let structure = RootMessage::new();
    Ok(serde_json::to_string(&structure)?)
}

#[no_mangle]
pub extern fn get_schema_url() -> String{
    return include_str!("../schema_url.txt").to_string();
}

// This should be replaced with a way to query the RPC.
#[no_mangle]
pub extern fn get_non_standard_library_interface_functions() -> Vec<String> {
    let ret = Vec::new();
    //ret.push("non_standard_function");
    ret
}

///////// Non-Standard //////////