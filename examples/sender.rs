extern crate failure;
extern crate protocols;
extern crate serde_json;
extern crate rand;

use serde_json::json;
use std::net::UdpSocket;
use protocols::pluginhandler::PluginHandler;
use rand::{thread_rng, Rng};

fn main() -> Result<(), failure::Error> {
    use std::path::PathBuf;

    // Initialize handler
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("../../../libraries/test-protocol/target/debug/test_protocol.dll"))?;
    handler.load_plugin(&PathBuf::from("../../../libraries/root-message/target/debug/root_message.dll"))?;

    // Generate message
    let root_protocol_schema = include_str!("../libraries/root-message/schema_url.txt");
    let test_protocol_schema = include_str!("../libraries/test-protocol/schema_url.txt");

    // Currently, sending messages is long, tedious, and slow. This WILL change. Here are some possible changes:
    // Possible solution 1: (slow... lots of copying)
    //   let mut test_data = handler.generage_message_with_data(&[("name", "Test Name"),
    //                                                            ("data", "Test Data")]);
    //   let mut root_data = handler.generage_message_with_data(&[("schema_url",test_protocol_schema), 
    //                                                            ("unencrypted_message",test_data)]);
    //   // root_data would then be able to be sent.
    //
    // Possible solution 2: (fast, but relies on static libraries - which might be okay)
    //   Just use static libraries and generate the messages directly. What if we don't have static libraries?
    // Possible solution 3: (CAN work, but will cause runtime errors if library doesn't support the function call)
    //   let test_data = handler.get_plugin(test_protocol_schema).call_function("generate_default_test_message", Args("MyName"));
    //   let root_data = handler.get_plugin(root_protocol_schema).call_function("generate_with_submessage", Args(test_data));
    //   // functions given as a string such as generate_default_test_message would take just enough args to generate bare raw message.
    //   // These functions could also be exposed over rpc
    // Possible solution 4: 
    //   A combination of all of these solutions depending on situation.

    // Generate default json data messages that we can manipulate
    println!("Getting default message...");
    let mut root_data = handler.get_default_json_message(root_protocol_schema)?;
    let mut test_data = handler.get_default_json_message(test_protocol_schema)?;

    // In the future, there may be a better way to do this such as custom library functions.
    println!("Constructing message...");
    test_data["name"] = json!("Test Name");
    test_data["data"] = json!("Test Data");
    root_data["schema_url"] = json!(test_protocol_schema); // Add the schema of the sub-protocol.
    root_data["unencrypted_message"] = json!(test_data.to_string().as_bytes());     // Set the data to be that of the sub-protocol.

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