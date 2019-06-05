use lazy_static::lazy_static;
use protocols::Transporter::TransportNode;

pub mod autogen_protobuf;
pub mod test_interface;
// __PUBMODPROTOCOLS__ Do not remove this line. This line is used to add new protocols.

lazy_static! {
    static ref NODE: TransportNode = {
        let mut m = TransportNode::default();
        m.add_interface(test_interface::Interface{});
        // __REGISTERINTERFACES__ Do not remove this line. This line is used to add new protocols.
        m
    };
}

// Need this for some reason to compile to wasm.
pub fn main() { }

#[no_mangle]
pub extern fn init() {
    println!("Initializing Logger...");
    //log::trace!("Inside dynamic library init()...");
    if let Err(e) = protocols::logging::initialize_standard_logging("TEST-PROTOCOL|\t") {
        println!("{:?}", e); // What are we going to do - log it? lol
    }
    log::trace!("..Finished initializing logger!");
    println!("...Finished initializing logger!");
}

#[cfg(not(target_arch = "wasm32"))] 
#[no_mangle]
/// Messages are sent through a single "handle_request_ffi" function as bytes. 
/// These bytes represent a RequestTransport structure upon receive, and a ReturnTransport structure upon return.
pub extern fn handle_request_ffi(data: &[u8]) -> Vec<u8> {
    println!("handle_request_ffi");
    log::trace!("Inside dynamic library handle_request_ffi(...)...");
    let ret = protocols::pluginhandler::ffi_handle_received_bytes(&NODE, data);
    log::trace!("...Leaving dynamic library handle_request_ffi(...)");
    ret
}

// Define a function that is imported into the module.
extern "C" {
    #[cfg(target_arch = "wasm32")] 
    fn return_data(ptr: *const u8, len: usize, id: i32);
    #[cfg(target_arch = "wasm32")] 
    fn get_arg_data(ptr: *const u8, len: usize, id: i32);
    #[cfg(target_arch = "wasm32")] 
    fn test();
}

#[cfg(target_arch = "wasm32")] 
#[no_mangle]
/// Messages are sent through a single "handle_request_ffi" function as bytes. 
/// These bytes represent a RequestTransport structure upon receive, and a ReturnTransport structure upon return.
pub extern fn handle_request_ffi_wasm(id: i32, arg_bytesize: i32) -> i32 {
    println!("handle_request_ffi_wasm");
    println!("Inside dynamic library handle_request_ffi(...)...");
    log::trace!("Inside dynamic library handle_request_ffi(...)...");
    let arg_data = Vec::with_capacity(arg_bytesize as usize);
    unsafe { test(); }
    unsafe { get_arg_data(arg_data.as_ptr(), arg_bytesize as usize, id); }
    println!("Got arg {:?}", arg_data);
    log::trace!("Got arg {:?}", arg_data);
    let ret = protocols::pluginhandler::ffi_handle_received_bytes(&NODE, &arg_data);
    unsafe { return_data(ret.as_ptr(), ret.len(), id); }
    println!("...Leaving dynamic library handle_request_ffi(...)");
    log::trace!("...Leaving dynamic library handle_request_ffi(...)");
    return id;
}
