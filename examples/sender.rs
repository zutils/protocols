extern crate failure;
extern crate protocols;

use std::net::UdpSocket;
use protocols::pluginmanager::PluginManager;

fn main() -> Result<(), failure::Error> {
    // Initialize manager
    let manager = PluginManager::new();
    manager.load_single_plugin("./libraries/test-protocol/target/debug/test_protocol.dll")?;
    manager.load_single_plugin("./libraries/root-message/target/debug/root_message.dll")?;

    // Generate message
    let test_protocol_hash = include_str!("../libraries/root-message/hash.txt");
    let data = manager.get_default_message(test_protocol_hash)?;

    // Send to localhost. Use the receiver binary to receive this data.
    println!("Sending: {:?}", data);
    let socket = UdpSocket::bind("127.0.0.1:61945")?; // I chose a random port # It doesn't matter.
    socket.send_to(data.as_bytes(), "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}