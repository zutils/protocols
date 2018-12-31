#![allow(non_snake_case)]

use crate::transport_autogen::transport::{ Schema, Schema_oneof_data, Data, GenerateMessageInfo, RpcData};

use failure::Error;

pub trait AsStringer {
    fn as_string(&self) -> &str;
}

impl AsStringer for Schema {
    fn as_string(&self) -> &str {
        match self.data {
            Some(Schema_oneof_data::URL(ref m)) => m,
            Some(Schema_oneof_data::Ipfs(ref m)) => m,
            Some(Schema_oneof_data::Ipns(ref m)) => m,
            None => "",
        } 
    }
}

pub trait ToDataConverter: protobuf::Message {
    fn to_Data(&self, schema: &Schema) -> Result<Data, Error> {
        let serialized_data = self.write_to_bytes()?;

        let mut ret = Data::new();
        ret.set_schema(schema.clone());
        ret.set_serialized_data(serialized_data);
        Ok(ret)
    }
}

impl<T> ToDataConverter for T where T: protobuf::Message {}

pub fn schema_ipfs_from_str(schema_str: &str) -> Schema {
    let mut schema = Schema::new();
    schema.set_Ipfs(schema_str.to_string());
    schema
}

pub trait FromDataConverter {
    fn unwrap<T: protobuf::Message>(&self) -> Result<(Schema, T), Error>;
}

impl FromDataConverter for Data {
    fn unwrap<T: protobuf::Message>(&self) -> Result<(Schema, T), Error> {
        let schema = self.get_schema();
        let serialized_data = self.get_serialized_data();
        Ok((schema.to_owned(), protobuf::parse_from_bytes(serialized_data)?))
    }
}

/// Helper function for creation of GenerateMessageInfo
/// Yes - I know that we are taking Vecs instead of [u8]s. This is to prevent complex cloning.
pub fn generate_message_info(schema: Schema, template: &str, args: Vec<Vec<u8>>) -> GenerateMessageInfo {
    let mut generation = GenerateMessageInfo::default();
    generation.args = protobuf::RepeatedField::from_vec(args);
    generation.set_template(template.to_string());
    generation.set_schema(schema);
    generation
}

/// Helper function for creation of RpcData
/// Yes - I know that we are taking a Vec instead of a [u8]. This is so that the function doesn't call to_vec().
pub fn generate_rpc(schema: Schema, method_name: &str, serialized_data: Vec<u8>) -> RpcData {
    let mut rpc = RpcData::default();
    rpc.set_method_name(method_name.to_string());
    rpc.set_schema(schema);
    rpc.set_serialized_rpc_arg(serialized_data);
    rpc
}
