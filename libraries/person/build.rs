extern crate builder;
extern crate failure;
use failure::Error;

fn main() -> Result<(), Error> {
	use builder::*;

	let proto_file = "protos/person.proto";

	build_rust_code_from_protobuffer(proto_file)?;
	let hash = add_file_and_get_ipfs_hash(proto_file)?;
	println!("Got Hash: {}", hash);

	generate_default_lib_files(&hash, "person", "Person")?;

	Ok(())
}
