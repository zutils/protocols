extern crate failure;
extern crate protobuf_codegen_pure as pb;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate hyper;

use hyper::rt::Future;
use failure::Error;
use ipfs_api::IpfsClient;

use std::io::Write;

fn main() -> Result<(), Error> {
	let proto_file = "schema/rootmessage.proto";
	build_rust_code_from_protobuffer(proto_file)?;
	add_file_and_write_ipfs_hash(proto_file)?;
	
	Ok(())
}

/// Build rust code from protobuffer. 
fn build_rust_code_from_protobuffer(proto_filename: &str) -> Result<(), Error> {
	let mut customize = pb::Customize::default();
	customize.serde_derive = Some(true);

	let args = pb::Args {
			out_dir: "src",
			input: &[proto_filename],
			includes: &[""],
			customize
	};

	pb::run(args).expect("protoc");

	Ok(())
}

/// Adds the file to IPFS so that 1) we can get it's hash and 2) That any message expected to receive this hash knows the schema.
/// In parent program, lib.rs loads in the hash file at compile time so that it knows the hash.
fn add_file_and_write_ipfs_hash(proto_filename: &str) -> Result<(), Error> {
	use std::fs::File;	
	let client = IpfsClient::default();
	
	println!("Adding file to ipfs...");
	let file = File::open(proto_filename)?;
	let req = client.add(file)
					.map(move |result| { 
                        println!("Writing {} to hash.txt", result.hash);
                        if let Err(e) = write_to_file("./hash.txt", &result.hash) {
							println!("{:?}", e);
						}
                    })
					.map_err(|e| eprintln!("{}", e));

	hyper::rt::run(req);
    Ok(())
}

fn write_to_file(new_file: &str, contents: &str) -> Result<(), Error> {
	use std::fs::File;
	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}