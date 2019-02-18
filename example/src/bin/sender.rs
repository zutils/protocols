use std::net::UdpSocket;
use rand::Rng;
use selflib::test_autogen::test;

fn main() -> Result<(), failure::Error> {
    // Initialize logging
    protocols::logging::initialize_standard_logging("")?;

    let test = test::Test::new(Some("Test Name".into()), Some("Test Data".into()));

    let schema: protocols::SchemaIdentifier = include_str!("../../../libraries/test-protocol/schema_urls/test.txt").into();
    let rpc = protocols::utils::generate_rpc(schema, "ClientRPC/receive_test", quick_protobuf::serialize_into_vec(&test)?);

    // Connect to localhost server
    let port = rand::thread_rng().gen_range(1025, 65536);
    let port_str = port.to_string();
    let bind_address = "127.0.0.1:".to_string() + &port_str;
    log::info!("Connecting to {}", bind_address);
    let socket = UdpSocket::bind(bind_address)?; // I chose a random port # It doesn't matter.
    
    // Send to localhost. Use the receiver binary to receive this data.
    log::info!("Sending: {:?}", rpc);
    let send_data = quick_protobuf::serialize_into_vec(&rpc)?;
    socket.send_to(&send_data, "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}
