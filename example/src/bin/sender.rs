use std::net::UdpSocket;
use rand::Rng;
use protobuf::Message;
use selflib::test_autogen::test;

fn main() -> Result<(), failure::Error> {
    // Initialize logging
    protocols::utils::initialize_standard_logging("")?;

    let mut test = test::Test::new();
    test.set_name("Test Name".to_string());
    test.set_data("Test Data".to_string());

    let schema = protocols::utils::schema_ipfs_from_str(include_str!("../../../libraries/test-protocol/schema_urls/test.txt"));
    let rpc = protocols::utils::generate_rpc(schema, "ClientRPC/receive_test", test.write_to_bytes()?);

    // Connect to localhost server
    let port = rand::thread_rng().gen_range(1025, 65536);
    let port_str = port.to_string();
    let bind_address = "127.0.0.1:".to_string() + &port_str;
    log::info!("Connecting to {}", bind_address);
    let socket = UdpSocket::bind(bind_address)?; // I chose a random port # It doesn't matter.
    
    // Send to localhost. Use the receiver binary to receive this data.
    log::info!("Sending: {:?}", rpc);
    let send_data = base64::encode(&rpc.write_to_bytes()?);
    socket.send_to(send_data.as_bytes(), "127.0.0.1:23462")?; // Port 23462 aligns with the receive port in the receiver program

    Ok(())
}
