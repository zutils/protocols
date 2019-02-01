#![allow(non_snake_case)]

use crate::transport_autogen::transport::{ SchemaIdentifier, Data, RpcData};

use failure::Error;

impl ToString for SchemaIdentifier {
    fn to_string(&self) -> String {
        self.get_id().to_string()
    }
}

impl From<&str> for SchemaIdentifier {
    fn from(f: &str) -> SchemaIdentifier {
        let mut ret = SchemaIdentifier::new();
        ret.id = f.to_string();
        ret
    }
}

// TODO: Implement TryInto for things such as ToDataConverter

pub trait ToDataConverter: protobuf::Message {
    fn to_Data(&self, schema: &SchemaIdentifier) -> Result<Data, Error> {
        let serialized_data = self.write_to_bytes()?;

        let mut ret = Data::new();
        ret.set_schema(schema.clone());
        ret.set_serialized_data(serialized_data);
        Ok(ret)
    }
}

impl<T> ToDataConverter for T where T: protobuf::Message {}

pub trait FromDataConverter {
    fn unwrap<T: protobuf::Message>(&self) -> Result<(SchemaIdentifier, T), Error>;
}

impl FromDataConverter for Data {
    fn unwrap<T: protobuf::Message>(&self) -> Result<(SchemaIdentifier, T), Error> {
        let schema = self.get_schema();
        let serialized_data = self.get_serialized_data();
        Ok((schema.to_owned(), protobuf::parse_from_bytes(serialized_data)?))
    }
}

/// Helper function for creation of RpcData
/// Yes - I know that we are taking a Vec instead of a [u8]. This is so that the function doesn't call to_vec().
pub fn generate_rpc(schema: SchemaIdentifier, method_name: &str, serialized_data: Vec<u8>) -> RpcData {
    let mut rpc = RpcData::default();
    rpc.set_method_name(method_name.to_string());
    rpc.set_schema(schema);
    rpc.set_serialized_rpc_arg(serialized_data);
    rpc
}

use std::sync::Mutex;
use std::thread::{self, ThreadId};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref TAB_HASH: Mutex<HashMap<ThreadId, usize>> = { std::sync::Mutex::new(HashMap::new()) };
}

pub fn initialize_standard_logging(log_prefix: &'static str) -> Result<(), Error> {
    use fern::colors::{Color, ColoredLevelConfig};

    let mut colors = ColoredLevelConfig::new();
    colors.error = Color::BrightRed;
    colors.warn = Color::BrightYellow;
    colors.info = Color::BrightGreen;
    colors.debug = Color::BrightMagenta;
    colors.trace = Color::BrightBlue;

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(move |out, message, record| {
            let message = format!("{}", message);
            let mut tab_hash = TAB_HASH.lock().unwrap();
            let tab_count: &mut usize = tab_hash.entry(thread::current().id()).or_default();
            
            // Remove the tabs prior to printing
            if message.starts_with("...") {
                *tab_count-=1;
            }

            let tabs: String = std::iter::repeat("| ").take(*tab_count).collect();
            let formatted = format!("{:?}\t{}{}", thread::current().id(), tabs, log_prefix);

            match record.level() {
                log::Level::Info => out.finish(format_args!("{}{}{}", formatted, colors.color(record.level()), message)),
                log::Level::Debug => out.finish(format_args!("{}{}", formatted, message)),
                log::Level::Trace => out.finish(format_args!("{}{}", formatted, message)),
                _ => out.finish(format_args!("{}{}{}", formatted, colors.color(record.level()), message)),
            }

            // Insert the tabs after printing
            if message.ends_with("...") {
                *tab_count+=1;
            }
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("mio", log::LevelFilter::Info)
        .level_for("tokio_reactor", log::LevelFilter::Info)
        .level_for("tokio_threadpool", log::LevelFilter::Info)
        .level_for("reqwest", log::LevelFilter::Info)
        .level_for("want", log::LevelFilter::Info)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply()?;

    log::trace!("Logging initialized!");
    Ok(())
}