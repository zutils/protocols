extern crate protobuf;
extern crate serde_json;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate failure;

pub mod rootmessage;
use rootmessage::RootMessage as TargetStructure;
use failure::Error;

/// String is the hash of another protocol. The Vec is the actual data.
pub struct SubmessageData(String, Vec<u8>);

#[no_mangle]
pub extern fn get_name() -> String {
    return "Rootmessage".to_string();
}

#[no_mangle]
pub extern fn handle(data: &[u8]) -> Result<Vec<SubmessageData>, Error> {
    let string: String = data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {}", string);
    let structure: TargetStructure = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);

    // Return the schema location and unencrypted message so that can be handled separately.
    let mut ret = Vec::new();
    ret.push(SubmessageData(structure.get_schema_location().to_string(), structure.get_unencrypted_message().to_vec()));
    Ok(ret)
}

#[no_mangle]
pub extern fn generate_message(_template_name: &str) -> Result<String, Error> {
    // for now, just generate a default message.
    let structure = TargetStructure::new();
    Ok(serde_json::to_string(&structure)?)
}

#[no_mangle]
pub extern fn get_hash() -> String{
    return include_str!("../hash.txt").to_string();
}
