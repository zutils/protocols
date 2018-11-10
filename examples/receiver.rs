extern crate failure;
extern crate protocols;

use std::net::UdpSocket;
use protocols::pluginhandler::*;

fn main() -> Result<(), failure::Error> {
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    // Initialize handler
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("../../../libraries/test-protocol/target/debug/test_protocol.dll"))?;
    let _test_protocol_schema = include_str!("../libraries/test-protocol/schema_urls/test.txt");
    let root_protocol_schema = include_str!("../libraries/test-protocol/schema_urls/root.txt");

    // Receive a message from the sender crate.
    let socket = UdpSocket::bind("127.0.0.1:23462")?; // I chose a random port #
    let mut buf = [0; 1024];
    let (byte_count, sender_ip) = socket.recv_from(&mut buf)?;
    println!("Received {} bytes from {}", byte_count, sender_ip);

    // Handle message received.  
    // The submessage of the root-message will be a test-protocol message. If it is loaded in the handler, it will be handled.
    let msg = MessageInfo::new(Vec::new(), &root_protocol_schema, &buf[0..byte_count]);
    handler.handle_msg_and_submsgs(msg)?;

    // handle_msg_and_submsgs(...) spawns a thread. If we do not sleep, the program will exit before thread handles the messages.
    thread::sleep(Duration::from_millis(500));

    Ok(())
}