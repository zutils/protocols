extern crate failure;
extern crate protocols;
extern crate serde_json;
extern crate rand;
extern crate libloading as lib;

use std::net::UdpSocket;

use protocols::pluginhandler::{PluginHandler, FFILibraryHashMapValue};
use rand::Rng;
use failure::Error;

// Future: Replace these with a macro of some sort.
fn generate_root_message(plugin: &FFILibraryHashMapValue, schema_url: &str, data: &[u8]) -> Result<String, Error> {
    println!("Nonstandard function: Generating root message for {}...", schema_url);
    let plugin = plugin.lock().unwrap();
    unsafe {
        let func: lib::Symbol<unsafe extern fn(&str, &[u8]) -> Result<String, Error>> = 
                    plugin.get_library()?.get(b"generate_root_message").expect("generate_message function not found in library!");
        func(schema_url, data)
    }
}

fn generate_test_message(plugin: &FFILibraryHashMapValue, name: &str, data: &str) -> Result<String, Error> {
    println!("Nonstandard function: Generating test message {}...", name);
    let plugin = plugin.lock().unwrap();
    unsafe {
        let func: lib::Symbol<unsafe extern fn(&str, &str) -> Result<String, Error>> = 
                    plugin.get_library()?.get(b"generate_test_message").expect("generate_message function not found in library!");
        func(name, data)
    }
}

fn main() -> Result<(), Error> {
    use std::path::PathBuf;

    // Initialize plugin handler
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("../../../libraries/test-protocol/target/debug/test_protocol.dll"))?;

    let test_protocol_schema = include_str!("../libraries/test-protocol/schema_urls/test.txt");
    let _root_protocol_schema = include_str!("../libraries/test-protocol/schema_urls/root.txt");

    // Calling non-standard functions for message generation
    let plugin = handler.get_plugin(&test_protocol_schema)?; // We know this is the same plugin as the root protocol
    let test_data = generate_test_message(&plugin, "Test Name", "Test Data")?;
    let root_data = generate_root_message(&plugin, &test_protocol_schema, test_data.as_bytes())?;
    
    // Send to localhost. Use the receiver binary to receive this data.
    println!("Sending: {:?}", root_data);

    let port = rand::thread_rng().gen_range(1025, 65536);
    let port_str = port.to_string();
    let bind_address = "127.0.0.1:".to_string() + &port_str;
    println!("Connecting to {}", bind_address);
    let socket = UdpSocket::bind(bind_address)?; // I chose a random port # It doesn't matter.
    socket.send_to(root_data.to_string().as_bytes(), "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}