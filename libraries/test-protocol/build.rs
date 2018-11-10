extern crate failure;
extern crate protocols;

use protocols::buildfunctions;
use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_file: &str) -> Result<(), Error> {
	buildfunctions::build_rust_code_from_protobuffer(proto_file)?;
	buildfunctions::add_file_and_write_ipfs_hash(&PathBuf::from(proto_file))?;
	Ok(())
}

fn main() -> Result<(), Error> {
	create_protobuf("./schema/test.proto")?;
	create_protobuf("./schema/root.proto")?;
	Ok(())
}
