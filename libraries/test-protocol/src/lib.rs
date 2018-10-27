extern crate protobuf;
extern crate serde_json;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate failure;

pub mod test;
use test::Test as TargetStructure;
use failure::Error;

/// String is the hash of another protocol. The Vec is the actual data.
pub struct SubmessageData(String, Vec<u8>);

#[no_mangle]
pub extern fn get_name() -> String {
    return "Test".to_string();
}

#[no_mangle]
pub extern fn handle(data: &[u8]) -> Result<Vec<SubmessageData>, Error> {
    let string: String = data.iter().map(|u: &u8| *u as char).collect();
    println!("Handling: {:?}", data);
    let structure: TargetStructure = serde_json::from_str(&string)?;
    println!("Received message: {:?}", structure);
    
    Ok(Vec::new())
}

#[no_mangle]
pub extern fn generate_message(template_name: &str) -> Result<String, Error> {
    // For now, just generate a default message
    let structure = TargetStructure::new();
    Ok(serde_json::to_string(&structure)?)
}

#[no_mangle]
pub extern fn get_hash() -> String{
    return include_str!("../hash.txt").to_string();
}
