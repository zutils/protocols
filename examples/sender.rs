extern crate failure;
extern crate protocols;
#[macro_use] extern crate serde_json;

use std::net::UdpSocket;
use protocols::pluginmanager::PluginManager;

fn main() -> Result<(), failure::Error> {
    // Initialize manager
    let manager = PluginManager::new();
    manager.load_single_plugin("./libraries/test-protocol/target/debug/test_protocol.dll")?;
    manager.load_single_plugin("./libraries/root-message/target/debug/root_message.dll")?;

    // Generate message
    let root_protocol_hash = include_str!("../libraries/root-message/hash.txt");
    let test_protocol_hash = include_str!("../libraries/test-protocol/hash.txt");

    // Currently, sending messages is long, tedious, and slow. This WILL change. Here are some possible changes:
    // Possible solution 1: (slow... lots of copying)
    //   let mut test_data = manager.generage_message_with_data(&[("name", "Test Name"),
    //                                                            ("data", "Test Data")]);
    //   let mut root_data = manager.generage_message_with_data(&[("schema_location",test_protocol_hash), 
    //                                                            ("unencrypted_message",test_data)]);
    //   // root_data would then be able to be sent.
    //
    // Possible solution 2: (fast, but relies on static libraries - which might be okay)
    //   Just use static libraries and generate the messages directly. What if we don't have static libraries?
    // Possible solution 3: (CAN work, but will cause runtime errors if library doesn't support the function call)
    //   let test_data = manager.get_plugin(test_protocol_hash).call_function("generate_default_test_message", Args("MyName"));
    //   let root_data = manager.get_plugin(root_protocol_hash).call_function("generate_with_submessage", Args(test_data));
    //   // functions given as a string such as generate_default_test_message would take just enough args to generate bare raw message.
    //   // These functions could also be exposed over rpc
    // Possible solution 4: 
    //   A combination of all of these solutions depending on situation.

    // Generate default json data messages that we can manipulate
    println!("Getting default message...");
    let mut root_data = manager.get_default_json_message(root_protocol_hash)?;
    let mut test_data = manager.get_default_json_message(test_protocol_hash)?;

    // In the future, there may be a better way to do this such as custom library functions.
    println!("Constructing message...");
    test_data["name"] = json!("Test Name");
    test_data["data"] = json!("Test Data");
    root_data["schema_location"] = json!(test_protocol_hash); // Add the hash of the sub-protocol.
    root_data["unencrypted_message"] = json!(test_data.to_string().as_bytes());     // Set the data to be that of the sub-protocol.

    // Send to localhost. Use the receiver binary to receive this data.
    println!("Sending: {:?}", root_data);
    let socket = UdpSocket::bind("127.0.0.1:61945")?; // I chose a random port # It doesn't matter.
    socket.send_to(root_data.to_string().as_bytes(), "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}