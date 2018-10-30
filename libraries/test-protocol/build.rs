extern crate failure;
extern crate protocols;

use protocols::buildfunctions;
use failure::Error;

fn main() -> Result<(), Error> {
	let proto_file = "schema/test.proto";
	buildfunctions::build_rust_code_from_protobuffer(proto_file)?;
	buildfunctions::add_file_and_write_ipfs_hash(proto_file)?;
	
	Ok(())
}
