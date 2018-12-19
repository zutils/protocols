use std::net::UdpSocket;
use std::path::PathBuf;

use protocols::pluginhandler::{PluginHandler, DynamicLibraryLoader};
use protocols::transport_autogen::transport::{GenerateMessageInfo, Schema};
use protocols::transport_autogen::transport_glue::ModuleToTransportGlue;
//use protocols::core;

fn main() -> Result<(), failure::Error> {
    use rand::Rng;
    use protobuf::RepeatedField;
    use protobuf::Message;

    // Initialize plugin handler. The PluginHandler is ALSO our module root.
    let handler = PluginHandler::new();
    handler.load_plugin(&PathBuf::from("./libraries/test-protocol/target/debug/test_protocol.dll"))?;

    // Create a schema so the handler knows what module to call.
    let test_schema = include_str!("../libraries/test-protocol/schema_urls/test.txt");
    let mut schema = Schema::new();
    schema.set_Ipfs(test_schema.to_string());

    // Create a GenerateMessageInfo structure and pass on to a function call
    let mut generation = GenerateMessageInfo::default();
    generation.args = RepeatedField::from_vec(vec![b"Test Name".to_vec(), b"Test Data".to_vec()]);
    generation.set_template("Test".to_string());
    generation.set_schema(schema);

    // Propogate through the handler tree to find a module matching the schema, and pass the generation info to it.
    let test_data = handler.generate_message(generation)?;

    // Send to localhost. Use the receiver binary to receive this data.
    println!("Sending: {:?}", test_data);

    let port = rand::thread_rng().gen_range(1025, 65536);
    let port_str = port.to_string();
    let bind_address = "127.0.0.1:".to_string() + &port_str;
    println!("Connecting to {}", bind_address);
    let socket = UdpSocket::bind(bind_address)?; // I chose a random port # It doesn't matter.
    socket.send_to(&test_data.write_to_bytes()?, "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}