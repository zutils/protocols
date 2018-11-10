extern crate protobuf_codegen_pure as pb;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate hyper;
extern crate toml;
extern crate toml_query;
extern crate failure;

use self::failure::Error;
use std::fs::File;	
use std::path::PathBuf;

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
						let schema_url = "https://ipfs.io/".to_string() + &result.hash;
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
		panic!(r#"Unable to retrieve schema URL. Make sure that IPFS daemon is running! You can get IPFS from ipfs.io"#);
	}

    Ok(())
}

fn write_to_file(new_file: &str, contents: &str) -> Result<(), Error> {
	use std::io::Write;

	println!("Writing file: {}", new_file);
	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}
