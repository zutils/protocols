#![allow(non_snake_case)]

use crate::transport_autogen::transport::{ SchemaIdentifier, Data, RpcData};

use std::convert::TryFrom;

impl<'a> TryFrom<SchemaIdentifier> for String {
    type Error = failure::Error;
    fn try_from(f: SchemaIdentifier) -> Result<String, Self::Error> {
        let err = failure::format_err!("Failure converting {:?} to string.", f);
        Ok(f.id.ok_or(err)?.to_string())
    }
}

impl<'a> From<&str> for SchemaIdentifier {
    fn from(f: &str) -> SchemaIdentifier {
        SchemaIdentifier::new(Some(f.into()))
    }
}

impl<'a, T> TryFrom<(Box<T>, SchemaIdentifier)> for Data where T: quick_protobuf::MessageWrite {
    type Error = failure::Error;

    fn try_from(from: (Box<T>, SchemaIdentifier)) -> Result<Data, Self::Error> {
        Ok(Data {
            schema: Some(from.1.clone()),
            serialized_data: Some(quick_protobuf::serialize_into_vec(&*from.0)?),
            sender: None,
        })
    }
}

/*pub trait FromDataConverter {
    fn unwrap<'a, T: quick_protobuf::MessageRead<'a> >(self) -> Result<(SchemaIdentifier, T), Error>;
}

impl FromDataConverter for Data {
    fn unwrap<'a, T: quick_protobuf::MessageRead<'a> >(self) -> Result<(SchemaIdentifier, T), Error> {
        let schema = self.schema.ok_or(failure::format_err!("No Schema!"))?.clone();
        let serialized_data = match self.serialized_data {
            Some(data) => data,
            None => return Err(failure::format_err!("No serialized data!")),
        };
        let deserialized_data = quick_protobuf::deserialize_from_slice(&serialized_data)?;
        Ok((schema, deserialized_data))
    }
}*/


/// Helper function for creation of RpcData
/// Yes - I know that we are taking a Vec instead of a [u8]. This is so that the function doesn't call to_vec().
pub fn generate_rpc<'a>(schema: SchemaIdentifier, method_name: &str, serialized_data: Vec<u8>) -> RpcData {
    RpcData {
        method_name: Some(method_name.into()),
        schema: Some(schema),
        serialized_rpc_arg: Some(serialized_data.into()),
        ..Default::default()
    }
}
