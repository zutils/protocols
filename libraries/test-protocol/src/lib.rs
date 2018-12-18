//#[macro_use] extern crate serde_derive;

use lazy_static::lazy_static;
use protocols::transport::TransportationNode;
use protocols::transmission_interface::transmission::{VecTransmission};

pub mod test_interface;
// __PUBMODPROTOCOLS__ Do not remove this line. This line is used to add new protocols.

// Warning! I am using unwrap here. The module will crash with unwrap() if an error is returned.
lazy_static! {
    static ref NODE: TransportationNode = {
        let mut m = TransportationNode::default();
        m.add_interface(test_interface::TestInterface{});
        // __REGISTERINTERFACES__ Do not remove this line. This line is used to add new protocols.
        m
    };
}

#[no_mangle]
/// Messages are sent through a single "propagate_ffi" function as bytes. 
/// These bytes represent a Transmission structure upon receive, and a VecTransmission structure upon return.
pub extern fn propagate_ffi(data: &[u8]) -> Vec<u8> {
    use protobuf::Message;
    use protocols::transport::Propagator;

    println!("Inside dynamic library propagate(...)");
    let mut ret = VecTransmission::new();

    match protocols::pluginhandler::parse_data_as_transmission(data) {
        Err(e) => {
            let transmission = protocols::pluginhandler::create_error_transmission(
                &format!("Cannot parse data! Possibly incorrect version. {:?}", e));
            ret.vec.push(transmission);
        },
        Ok(transmission) => {
            let vectransmission_data = NODE.propagate_transmission(&transmission);
            ret = protocols::pluginhandler::to_vectransmission(vectransmission_data);
        },
    } 

    // Because we cannot return a Result, we must fail :(
    match ret.write_to_bytes() {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            println!("Cannot write VecTransmission to bytes! {:?}", e);
            Vec::new() // Return NOTHING :( TODO: Write test case for this.
        }
    }
}
