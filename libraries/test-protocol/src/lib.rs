#[macro_use] extern crate serde_derive;
extern crate lazy_static;
extern crate failure;
extern crate protocols;
extern crate protobuf;

use failure::{Error, format_err};
use protocols::pluginhandler::{MessageInfo, SubLibrary};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub mod root;
pub mod test;
pub mod root_interface;
pub mod test_interface;
use root_interface::RootInterface;
use test_interface::TestInterface;

type SubLibraryKeyValue = Box<SubLibrary+Sync>;

lazy_static! {
    static ref SUBLIBRARIES: HashMap<String, SubLibraryKeyValue> = {
        let mut m: HashMap<String, SubLibraryKeyValue> = HashMap::new();
        m.insert(RootInterface::get_schema_url(), Box::new(RootInterface{}));
        m.insert(TestInterface::get_schema_url(), Box::new(TestInterface{}));
        m
    };
}

#[no_mangle]
/// Pass function through to hashmap
pub extern fn get_name(schema_url: &str) -> Result<String, Error> {
    let library = SUBLIBRARIES.get(schema_url).ok_or(format_err!("{} does not exist in this library!", schema_url))?;
    Ok(library.get_name())
}

#[no_mangle]
/// Pass function through to hashmap
pub extern fn handle(info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
    println!("Handling {} message in library.", info.schema_url);
    let library = SUBLIBRARIES.get(&info.schema_url).ok_or(format_err!("{} does not exist in this library!", info.schema_url))?;
    library.handle(info)
}

#[no_mangle]
/// Return list of all schema urls
pub extern fn get_schema_urls() -> Result<Vec<String>, Error> {
    Ok(SUBLIBRARIES.keys().map(|s| s.clone()).collect::<Vec<_>>())
}

// This should be replaced with a way to query the RPC.
/*#[no_mangle]
pub extern fn get_nonstandard_library_interface_functions() -> Vec<&'static str> {
    vec!["generate_root_message(schema_url: &str, data: &[u8]) -> Result<String, Error>"]
}*/
