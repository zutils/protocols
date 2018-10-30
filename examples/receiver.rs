extern crate failure;
extern crate protocols;

use std::net::UdpSocket;
use protocols::pluginmanager::PluginManager;

fn main() -> Result<(), failure::Error> {
    // Initialize manager
    let manager = PluginManager::new();
    manager.load_single_plugin("./libraries/test-protocol/target/debug/test_protocol.dll")?;
    manager.load_single_plugin("./libraries/root-message/target/debug/root_message.dll")?;

    // Receive a message from the sender crate.
    let socket = UdpSocket::bind("127.0.0.1:23462")?; // I chose a random port #
    let mut buf = [0; 1024];
    let (_byte_count, _sender_ip) = socket.recv_from(&mut buf)?;

    // Handle message received.  
    // The submessage of the root-message will be a test-protocol message. If it is loaded in the manager, it will be handled.
    let root_message_hash = include_str!("../libraries/root-message/schema_url.txt").to_string();
    manager.handle_msg_and_submsgs(&root_message_hash, &buf[0.._byte_count])?;

    Ok(())
}