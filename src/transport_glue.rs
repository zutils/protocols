#![allow(non_snake_case)]

use crate::common::*;
use crate::autogen_protobuf::transport::*;

use failure::Error;

// Anywhere there is a common model or structure, we want transports working with them.
impl<T> TransportToModelGlue for T where T: CommonModelFunctions {}
impl<T> TransportToProcessorGlue for T where T: CommonStructureFunctions {}

/// These functions are the endpoints to the different modules.
pub trait TransportToProcessorGlue: CommonStructureFunctions {
    fn handle_transport(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        // Pass on transport to proper function
        let ret_data = match &transport.event.data {
            mod_Event::OneOfdata::process_struct(arg) => self.process_struct(arg.clone())?,
            other => return Err(failure::format_err!("{:?} request function type unsupported!", other)),
        };
        Ok(ret_data.into())
    }
}

/// These functions are the endpoints to the different modules.
pub trait TransportToModelGlue: CommonModelFunctions {
    fn handle_transport(&mut self, transport: &RequestTransport) -> Result<ReturnTransport, Error> {
        // Pass on transport to proper function
        let ret_data = match &transport.event.data {
            mod_Event::OneOfdata::constructor(arg) => self.constructor(arg.clone())?,
            mod_Event::OneOfdata::destructor(arg) => self.destructor(arg.clone())?,
            mod_Event::OneOfdata::update_model(arg) => self.update_model(arg.clone())?,
            other => return Err(failure::format_err!("{:?} request function type unsupported!", other)),
        };
        Ok(ret_data.into())
    }
}


impl From<Vec<Event>> for ReturnTransport {
    fn from(f: Vec<Event>) -> ReturnTransport {
        ReturnTransport::new(f, vec![])
    }
}

impl From<String> for ReturnTransport {
    fn from(f: String) -> ReturnTransport {
        ReturnTransport::new(vec![], vec![f])
    }
}

impl From<ReturnTransport> for Vec<Event> {
    fn from(f: ReturnTransport) -> Vec<Event> {
        // Don't return errors, there may be valid data... print them out.
        for err in f.errors { log::warn!("{:?}", err); }
        f.vec
    }
}
