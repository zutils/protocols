#[macro_use] extern crate serde_derive;

use failure::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub mod test;
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
    println!("Inside dynamic library propagate(...)");
    let mut ret = VecTransmission::new();

    match parse_data_as_transmission(data) {
        Err(e) => {
            let transmission = create_error_transmission(format!("Cannot parse data! Possibly incorrect version. {:?}", e));
            ret.vec.push(transmission);
        },
        Ok(transmission) => {
            ret = NODE.propagate_transmission(transmission).to_VecTransmission();
        },
    } 

    let bytes = vectransmission.write_to_bytes()?;
    bytes.to_vec()
}
