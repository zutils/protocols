extern crate protobuf_codegen_pure as pb;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate failure;
extern crate hyper;

use hyper::rt::Future;
use failure::Error;
use ipfs_api::IpfsClient;

use std::env;
use std::io::Write;

const PROTO_FILE: &str = "protos/rootmessage.proto";

fn main() -> Result<(), Error> {
	let p = env::current_dir().unwrap();
	println!("The current directory is {}", p.display());

	let mut customize = pb::Customize::default();
	customize.serde_derive = Some(true);

	let args = pb::Args {
			out_dir: "src",
			input: &[PROTO_FILE],
			includes: &[""],
			customize
	};

	// Send the .proto file to ipfs and record the hash in a file for compilation. That hash will be added to the compiled code.
	use std::fs::{File};
	let client = IpfsClient::default();
	
	println!("Adding file to ipfs...");
	let file = File::open(PROTO_FILE)?;
	let req = client.add(file)
					.map(|result| {
						// Write the hash of the proto file to a file for loading in the library code
						println!("Writing hash to hash.txt: {:?}", result.hash);
						let mut file = File::create("hash.txt").unwrap();
						file.write_all(result.hash.as_bytes()).unwrap();
					})
					.map_err(|e| eprintln!("{}", e));

	hyper::rt::run(req);
	pb::run(args).expect("protoc");

	Ok(())
}
