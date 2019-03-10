use crate::autogen::transport::{ SchemaIdentifier};

impl<'a> From<&str> for SchemaIdentifier {
    fn from(f: &str) -> SchemaIdentifier {
        SchemaIdentifier::new(f.to_string())
    }
}
