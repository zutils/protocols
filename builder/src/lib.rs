extern crate protobuf_codegen_pure as pb;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate failure;
extern crate hyper;

use hyper::rt::Future;
use failure::Error;
use ipfs_api::IpfsClient;

use std::io::Write;

/// You will need this function to generate base-code.
pub fn generate_default_lib_files(hash: &str, lib_file_base_name: &str, structure_name: &str) -> Result<(), Error> {
	use std::fs::{self, File};

	let mut lib_file_content = include_str!("./default_lib.txt").to_string();
	lib_file_content = lib_file_content.replace("__HASHCODE__", hash);
	lib_file_content = lib_file_content.replace("__LIB_FILE_BASE_NAME__", lib_file_base_name);
	lib_file_content = lib_file_content.replace("__STRUCTURE_NAME__", structure_name);
	
	fs::create_dir_all("./src")?;
	let mut lib_file = File::create("./src/lib.rs")?;
	lib_file.write_all(lib_file_content.as_bytes())?;

	Ok(())
}

/// Build rust code from protobuffer. 
pub fn build_rust_code_from_protobuffer(proto_filename: &str) -> Result<(), Error> {
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

/// Adds the file to IPFS so that we can get it's hash.
/// Write hash to file.
pub fn add_file_and_get_ipfs_hash(proto_filename: &str) -> Result<String, Error> {
	use std::fs::File;
	use std::sync::{Arc, Mutex};
	
	let client = IpfsClient::default();
	
	println!("Adding file to ipfs...");
	let return_string = Arc::new(Mutex::new(String::new()));
	let return_clone = return_string.clone();
	let file = File::open(proto_filename)?;
	let req = client.add(file)
					.map(move |result| { *return_clone.lock().unwrap() = result.hash; () } )
					.map_err(|e| eprintln!("{}", e));

	hyper::rt::run(req);

	let ret: &String = &*return_string.lock().unwrap();
	Ok(ret.clone())
}