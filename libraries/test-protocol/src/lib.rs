use lazy_static::lazy_static;
use protocols::propagator::TransportNode;

pub mod test_interface;
pub mod test_autogen;
// __PUBMODPROTOCOLS__ Do not remove this line. This line is used to add new protocols.

lazy_static! {
    static ref NODE: TransportNode = {
        let mut m = TransportNode::default();
        m.add_interface(test_interface::TestInterface{});
        // __REGISTERINTERFACES__ Do not remove this line. This line is used to add new protocols.
        m
    };
}

#[no_mangle]
pub extern fn init() {
    log::trace!("Inside dynamic library init()...");
    if let Err(e) = protocols::utils::initialize_standard_logging("TEST-PROTOCOL|\t") {
        println!("{:?}", e); // What are we going to do - log it? lol
    }
    log::trace!("...Leaving dynamic library init()");
}

#[no_mangle]
/// Messages are sent through a single "propagate_ffi" function as bytes. 
/// These bytes represent a Transport structure upon receive, and a VecTransport structure upon return.
pub extern fn propagate_ffi(data: &[u8]) -> Vec<u8> {
    log::debug!("Inside dynamic library.");
    protocols::pluginhandler::ffi_handle_received_bytes(&NODE, data)
}
