use crate::autogen_protobuf::transport::*;

// We want to use specific structures as keys for a HashMap
use std::hash::{Hash, Hasher};

impl Hash for ModuleId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

impl Eq for ModuleId {}


impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.val.hash(state);
    }
}

impl Eq for crate::Id {}


impl Hash for TypeDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.libraryAlias.hash(state);
        self.structure.hash(state);
    }
}

impl Eq for crate::TypeDescriptor {}