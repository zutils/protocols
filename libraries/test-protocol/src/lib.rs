//#[macro_use] extern crate serde_derive;

use lazy_static::lazy_static;
use protocols::core::TransportNode;

pub mod test_interface;
pub mod test_autogen;
// __PUBMODPROTOCOLS__ Do not remove this line. This line is used to add new protocols.

// Warning! I am using unwrap here. The module will crash with unwrap() if an error is returned.
lazy_static! {
    static ref NODE: TransportNode = {
        let mut m = TransportNode::default();
        m.add_interface(test_interface::TestInterface{});
        // __REGISTERINTERFACES__ Do not remove this line. This line is used to add new protocols.
        m
    };
}

#[no_mangle]
/// Messages are sent through a single "propagate_ffi" function as bytes. 
/// These bytes represent a Transport structure upon receive, and a VecTransport structure upon return.
pub extern fn propagate_ffi(data: &[u8]) -> Vec<u8> {
    println!("Inside dynamic library.");
    protocols::pluginhandler::ffi_handle_received_bytes(&NODE, data)
}
