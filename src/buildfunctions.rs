//! buildfunctions provides functions to be used in plugins' build.rs file.
use protoc_rust_grpc as prg;
use protobuf_codegen_pure as pcp;
use failure::Error;
use std::fs::File;	
use std::path::PathBuf;

/// Call protoc on protobuffer and create non-rpc code
pub fn build_rust_code_from_protobuffer(proto_filename: &PathBuf) -> Result<PathBuf, Error> {
	println!("Building protobuf for {:?}", &proto_filename);

	let path_str = proto_filename.to_str().ok_or(failure::format_err!("Cannot create str from PathBuf!"))?;

	let mut customize = pcp::Customize::default();
	customize.serde_derive = Some(true);

	let out_dir = out_dir(&proto_filename);
	std::fs::create_dir_all(out_dir.clone())?;

	let args = pcp::Args {
		out_dir: &pathbuf_to_string(&out_dir)?,
		input: &[path_str],
		includes: &["./schema"],
		customize
	};

	pcp::run(args).expect("protoc");

	let out_file = get_protobuf_generated_file(proto_filename);
	println!("Protoc ran on {:?} and created {:?}", proto_filename, out_file);

	Ok(out_file)
}

/// Call protoc on protobuffer and create only the rpc code
pub fn build_rust_rpc_code_from_protobuffer(proto_filename: &PathBuf) -> Result<PathBuf, Error> {
	println!("Building protobuf rpc for {:?}", &proto_filename);
	let path_str = proto_filename.to_str().ok_or(failure::format_err!("Cannot create str from PathBuf!"))?;

	let out_dir = out_dir(&proto_filename);

	let args = prg::Args {
		out_dir: &pathbuf_to_string(&out_dir)?,
		input: &[path_str],
		includes: &["./schema"],
		rust_protobuf: false,
		..Default::default()
	};

	prg::run(args).expect("protoc-rust-grpc");

	let out_file = get_protobuf_rpc_generated_file(proto_filename);
	println!("Protoc-rust-grpc ran on {:?} and created {:?}", proto_filename, out_file);

	Ok(out_file)
}

/// Adds the file to IPFS so that 1) we can get it's hash and 2) So that we can generate a schema url from that hash
/// In parent program, lib.rs loads in the schema_link at compile time so that the library can use it.
pub fn add_file_and_write_ipfs_hash(path: &PathBuf) -> Result<String, Error> {
	use hyper::rt::Future;
	use std::sync::{Arc, Mutex};
	let client = ipfs_api::IpfsClient::default();
	
	// Create atomics for hyper
	let should_panic = Arc::new(Mutex::new(false));
	let should_panic_clone = should_panic.clone();
	let hash = Arc::new(Mutex::new(String::new()));
	let hash_clone = hash.clone();

	println!("Adding {:?} to ipfs...", path);
	let file = File::open(path)?;
	let req = client
		.add(file)
		.map(move |result| { 
			let mut hash = hash_clone.lock().unwrap();
			*hash = result.hash;
		})
		.map_err(move |_e| {
			let mut data = should_panic_clone.lock().unwrap();
			*data = true; 
		});

	hyper::rt::run(req);

	// We have to panic in the main thread.
	if *should_panic.lock().unwrap() == true {
		panic!(r#"Unable to retrieve schema URL from ipfs. Make sure that IPFS daemon is running! You can get IPFS from ipfs.io\nIf you REALLY don't want to use ipfs, and care to handle the schema_link manually, modify your build.rs file."#);
	}

	let schema_link_file_location = format!("./schema_urls/{}.txt", base_name(path));
	let hash = hash.lock().unwrap().clone();
	write_to_file(&PathBuf::from(schema_link_file_location), &hash)?;

    Ok(hash)
}

pub fn modify_file(path: &PathBuf, pretext: &str, posttext: &str) -> Result<(), Error> {
	let file_data = ::std::fs::read_to_string(path)?;
	let modified_file = file_data.replace(pretext, posttext);
	write_to_file(path, &modified_file)?;
	Ok(())
}

pub fn for_all_in_dir(path_str: &str, func: fn(&PathBuf) -> Result<(), Error>) {
	use std::fs;
    let paths = fs::read_dir(path_str).unwrap();

    for path in paths {
		let path = path.unwrap().path();
		if let Err(e) = func(&path) {
			println!("{:?}", e);
		}
    }
}

pub fn write_to_file(new_file: &PathBuf, contents: &str) -> Result<(), Error> {
	use std::io::Write;

	println!("Writing file: {:?}", new_file);
	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}

fn get_protobuf_generated_file(proto_filename: &PathBuf) -> PathBuf {
	// Figure out the file that was generated.
	let base_name = base_name(proto_filename);
	let mut out_file = out_dir(&proto_filename);
	out_file.push(format!("{}.rs", base_name));
	out_file
}

fn get_protobuf_rpc_generated_file(proto_filename: &PathBuf) -> PathBuf {
	// Figure out the file that was generated.
	let base_name = base_name(proto_filename);
	let mut out_file = out_dir(&proto_filename);
	out_file.push(format!("{}_grpc.rs", base_name));
	out_file
}

fn base_name(protobuf_path: &PathBuf) -> String {
	let base_name: String = protobuf_path.file_stem().unwrap().to_str().unwrap().to_string();
	base_name
}

fn out_dir(protobuf_path: &PathBuf) -> PathBuf {
	let mut dir = PathBuf::from(std::env::current_dir().unwrap());
	dir.push("src");
	let base_name = base_name(protobuf_path);
	dir.push(format!("{}_interface", base_name));
	dir
}

fn pathbuf_to_string(path: &PathBuf) -> Result<String, Error> {
	Ok(path.to_str().ok_or(failure::format_err!("Cannot create str from PathBuf!"))?.to_string())
}