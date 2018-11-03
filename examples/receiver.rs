extern crate failure;
extern crate protocols;

use std::net::UdpSocket;
use protocols::pluginhandler::*;

fn main() -> Result<(), failure::Error> {
    use std::path::PathBuf;
    use std::thread;

    // Initialize handler
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("../../../libraries/test-protocol/target/debug/test_protocol.dll"))?;
    handler.load_plugin(&PathBuf::from("../../../libraries/root-message/target/debug/root_message.dll"))?;

    // Receive a message from the sender crate.
    let socket = UdpSocket::bind("127.0.0.1:23462")?; // I chose a random port #
    let mut buf = [0; 1024];
    let (byte_count, sender_ip) = socket.recv_from(&mut buf)?;
    println!("Received {} bytes from {}", byte_count, sender_ip);

    // Handle message received.  
    // The submessage of the root-message will be a test-protocol message. If it is loaded in the handler, it will be handled.
    let root_message_schema_url = include_str!("../libraries/root-message/schema_url.txt");
    let msg = MessageInfo::new(Vec::new(), root_message_schema_url, &buf[0..byte_count]);
    handler.handle_msg_and_submsgs(msg);

    // handle_msg_and_submsgs(...) spawns a thread. If we do not sleep, the program will exit before thread handles the messages.
    thread::sleep_ms(500);

    Ok(())
}