#![allow(non_snake_case)]

use crate::transport_autogen::transport::{ SchemaIdentifier, RpcData};

impl<'a> From<&str> for SchemaIdentifier {
    fn from(f: &str) -> SchemaIdentifier {
        SchemaIdentifier::new(f.to_string())
    }
}

/// Helper function for creation of RpcData
/// Yes - I know that we are taking a Vec instead of a [u8]. This is so that the function doesn't call to_vec().
pub fn generate_rpc(schema: SchemaIdentifier, method_name: &str, serialized_data: Vec<u8>) -> RpcData {
    RpcData {
        method_name: method_name.to_string(),
        schema: schema,
        serialized_rpc_arg: serialized_data,
        ..Default::default()
    }
}
