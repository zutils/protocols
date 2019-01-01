use std::net::UdpSocket;
use std::path::PathBuf;

use protocols::pluginhandler::{PluginHandler, DynamicLibraryLoader};
use protocols::{Data, ModuleToTransportGlue};

fn main() -> Result<(), failure::Error> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    // Initialize plugin handler. The PluginHandler is ALSO our module root.
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("./libraries/test-protocol/target/release/test_protocol.dll"))?;
    
    // Receive a message from the sender crate.
    let socket = UdpSocket::bind("127.0.0.1:23462")?; // I chose a random port #
    log::info!("Listening on {:?}", socket);
    let mut buf = [0; 1024];
    let (byte_count, sender_ip) = socket.recv_from(&mut buf)?;
    log::info!("Received {} bytes from {}", byte_count, sender_ip);

    // Convert received bytes to a Data type.
    let received_data = &buf[0..byte_count];
    let data: Data = protobuf::parse_from_bytes(&received_data)?; 

    // Propogate through the handler tree to find a module matching the schema, and pass the generation info to it.
    // Note that we do not pass in a schema for data as the data message already contains the schema it is supposed to be used for.
    handler.handle_trusted(data)?;

    Ok(())
}