extern crate protobuf_codegen_pure as pcp;
extern crate protoc_rust_grpc as prg;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate hyper;
extern crate toml;
extern crate toml_query;
extern crate failure;

use self::failure::{Error, format_err};
use std::fs::File;	
use std::path::PathBuf;

/// Call protoc on protobuffer and create non-rpc code
pub fn build_rust_code_from_protobuffer(proto_filename: &PathBuf) -> Result<(), Error> {
	let path_str = proto_filename.to_str().ok_or(format_err!("Cannot create str from PathBuf!"))?;

	let mut customize = pcp::Customize::default();
	customize.serde_derive = Some(true);

	let args = pcp::Args {
			out_dir: "src",
			input: &[path_str],
			includes: &["./schema"],
			customize
	};

	pcp::run(args).expect("protoc");

	Ok(())
}

/// Call protoc on protobuffer and create only the rpc code
pub fn build_rust_rpc_code_from_protobuffer(proto_filename: &PathBuf) -> Result<(), Error> {
	let path_str = proto_filename.to_str().ok_or(format_err!("Cannot create str from PathBuf!"))?;

	let args = prg::Args {
			out_dir: "src",
			input: &[path_str],
			includes: &["./schema"],
			rust_protobuf: false,
			..Default::default()
	};

	prg::run(args).expect("protoc-rust-grpc");

	Ok(())
}

/// Adds the file to IPFS so that 1) we can get it's hash and 2) So that we can generate a schema url from that hash
/// In parent program, lib.rs loads in the schema_url at compile time so that the library can use it.
pub fn add_file_and_write_ipfs_hash(path: &PathBuf) -> Result<(), Error> {
	use self::hyper::rt::Future;
	use std::sync::{Arc, Mutex};
	let client = ipfs_api::IpfsClient::default();
	
	println!("Adding file to ipfs...");
	let should_panic = Arc::new(Mutex::new(false));
	let should_panic_clone = should_panic.clone();
	let file = File::open(path)?;
	let base_name: String = path.file_stem().unwrap().to_str().unwrap().to_string(); // Create string so that we can add it to thread.
	let req = client.add(file)
					.map(move |result| { 
						let schema_url = result.hash;
						let schema_url_file_location = format!("./schema_urls/{}.txt", base_name);
                        write_to_file(&schema_url_file_location, &schema_url).unwrap();
                    })
					.map_err(move |_e| {
						let mut data = should_panic_clone.lock().unwrap();
						*data = true; 
					});

	hyper::rt::run(req);

	// We have to panic in the main thread.
	if *should_panic.lock().unwrap() == true {
		panic!(r#"Unable to retrieve schema URL from ipfs. Make sure that IPFS daemon is running! You can get IPFS from ipfs.io\nIf you REALLY don't want to use ipfs, and care to handle the schema_url manually, modify your build.rs file."#);
	}

    Ok(())
}

pub fn for_all_in_dir(path_str: &str, func: fn(&PathBuf) -> Result<(),Error>) {
	use std::fs;
    let paths = fs::read_dir(path_str).unwrap();

    for path in paths {
		let path = path.unwrap().path();
		println!("Building {:?}", &path);

		if let Err(e) = func(&path) {
			println!("{:?}", e);
		}
    }
}

fn write_to_file(new_file: &str, contents: &str) -> Result<(), Error> {
	use std::io::Write;

	println!("Writing file: {}", new_file);
	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}
