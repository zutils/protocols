use crate::autogen::transport::{ SchemaIdentifier};

impl From<&str> for SchemaIdentifier {
    fn from(f: &str) -> SchemaIdentifier {
        SchemaIdentifier::new(f.to_string())
    }
}

impl From<String> for SchemaIdentifier {
    fn from(f: String) -> SchemaIdentifier {
        SchemaIdentifier::new(f)
    }
}
