use std::net::UdpSocket;
use std::path::PathBuf;

use protocols::pluginhandler::{PluginHandler, DynamicLibraryLoader};
use protocols::{ModuleToTransportGlue};
use rand::Rng;
use protobuf::Message;

fn main() -> Result<(), failure::Error> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "trace");
    pretty_env_logger::init();

    // Initialize plugin handler. The PluginHandler is ALSO our module root.
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("./libraries/test-protocol/target/release/test_protocol.dll"))?;

    // Create a schema so the handler knows what module to call.
    let schema = protocols::utils::schema_ipfs_from_str(include_str!("../libraries/test-protocol/schema_urls/test.txt"));

    // Create a GenerateMessageInfo structure and pass on to a function call
    let generation = protocols::utils::generate_message_info(schema, "Test", vec![b"Test Name".to_vec(), b"Test Data".to_vec()]);

    // Propogate through the handler tree to find a module matching the schema, and pass the generation info to it.
    let test_data = handler.generate_message(generation)?;

    // Connect to localhost server
    let port = rand::thread_rng().gen_range(1025, 65536);
    let port_str = port.to_string();
    let bind_address = "127.0.0.1:".to_string() + &port_str;
    log::info!("Connecting to {}", bind_address);
    let socket = UdpSocket::bind(bind_address)?; // I chose a random port # It doesn't matter.
    
    // Send to localhost. Use the receiver binary to receive this data.
    log::info!("Sending: {:?}", test_data);
    socket.send_to(&test_data.write_to_bytes()?, "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}
