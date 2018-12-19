#![allow(non_snake_case)]

use crate::transport_autogen::transport::{ Schema, Schema_oneof_data, Data};
use crate::common::CommonModule;

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

impl<T> ToDataConverter for T where T: CommonModule + protobuf::Message {}

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

pub fn from_data<T: protobuf::Message>(data: &Data) -> Result<(Schema, T), Error> {
    let schema = data.get_schema();
    let serialized_data = data.get_serialized_data();
    Ok((schema.to_owned(), protobuf::parse_from_bytes(serialized_data)?))
}
