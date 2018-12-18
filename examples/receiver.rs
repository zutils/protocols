use std::net::UdpSocket;
use std::path::PathBuf;

use protocols::pluginhandler::PluginHandler;

fn main() -> Result<(), failure::Error> {
    // Initialize plugin handler. The PluginHandler is ALSO our module root.
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("./libraries/test-protocol/target/debug/test_protocol.dll"))?;
    
    // Get the schema URLs from the files. By default, create-protocols-plugin generates these in a file we can pull from :)
    let test_schema = Schema(include_str!("../libraries/test-protocol/schema_links/test.txt"));

    // Receive a message from the sender crate.
    let socket = UdpSocket::bind("127.0.0.1:23462")?; // I chose a random port #
    let mut buf = [0; 1024];
    let (byte_count, sender_ip) = socket.recv_from(&mut buf)?;
    println!("Received {} bytes from {}", byte_count, sender_ip);
    let received_data = &buf[0..byte_count];

    // Propogate through the handler tree to find a module matching the schema, and pass the generation info to it.
    // Note that we do not pass in a schema for data as the data message already contains the schema it is supposed to be used for.
    handler.handle_trusted(received_data)?;

    Ok(())
}