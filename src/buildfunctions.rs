//! buildfunctions provides functions to be used in plugins' build.rs file.
use failure::Error;
use std::fs::File;	
use std::path::PathBuf;
use std::io::Write;

pub fn generate_rpc_traits_and_handler<W: Write + ?Sized>(rpc: &pb_rs::types::RpcService, w: &mut W) -> Result<(), pb_rs::errors::Error> {
	// Box::new(|rpc, writer| generate_rpc_traits_and_handler(rpc, writer))
    /* Example:
        trait <service> {
            fn <func>(&self, arg: &<arg>) -> Result<<ret>, failure::Error>;
        }

		fn handle_PublicRPC(data: &RpcData) -> Result<VecRpcData, Error> {
			let serialized_arg = quick_protobuf::deserialize_from_slice(&data.serialized_rpc_arg)?;
			match data.method_name.as_ref() {
				"PublicRPC/publish_data" => PublicRPC::publish_data(serialized_arg),
				_ => Vec::new(),
			}
		}
    */

    writeln!(w, "\npub trait {SERVICE} {{", SERVICE = rpc.service_name)?;
	let ret = "Vec<RpcData>"; // Formerly func.ret for ACTUAL ret... it won't work with code though.
    for func in rpc.functions.iter() {
        writeln!(w, "   fn {FUNC}(&self, _arg: {ARG}) -> std::result::Result<{RET}, failure::Error> {{", 
            FUNC = func.name, ARG = func.arg, RET = ret)?; 
		writeln!(w, r#"		Err(failure::format_err!("No Rpc for {FUNC}!"))"#, FUNC = func.name)?;
		writeln!(w, "	}}")?;	
    }
    writeln!(w, "}}")?;

	let service_name = rpc.service_name.clone();
	writeln!(w, "pub fn handle_{SERVICE}<H: {SERVICE}>(data: &protocols::RpcData, {UNDERSCORE}handler: H) -> std::result::Result<protocols::VecRpcData, failure::Error> {{", 
		SERVICE = service_name, UNDERSCORE = if rpc.functions.is_empty() { "_" } else { "" } )?;
	if !rpc.functions.is_empty() {
		writeln!(w, "	match data.method_name.as_ref() {{")?;
		for func in rpc.functions.iter() {
			writeln!(w, r#"			"{SERVICE}/{FUNC}" => Ok(handler.{FUNC}(quick_protobuf::deserialize_from_slice(&data.serialized_rpc_arg)?)?.into()),"#, 
				SERVICE = service_name, FUNC = func.name)?;
		}
		writeln!(w, r#"		_ => Err(failure::format_err!("Cannot find rpc function {{}}", data.method_name)),"#)?;
		writeln!(w, "	}}\n")?;
	} else {
		writeln!(w, r#"		Err(failure::format_err!("Rpc is unsupported for {{:?}}. Cannot execute {{}}", data.schema, data.method_name))"#)?;
	}
	writeln!(w, "}}\n")?;

    Ok(())
}

pub fn build_rust_code_from_protobuffer(proto_filename: &PathBuf) -> Result<PathBuf, Error> {
	build_rust_code_from_protobuffer_with_options(proto_filename, vec!["use protocols::{Data, RpcData};".to_string()],
		Box::new(|rpc, writer| generate_rpc_traits_and_handler(rpc, writer)))
}

/// Call protoc on protobuffer and create non-rpc code
pub fn build_rust_code_from_protobuffer_with_options(proto_filename: &PathBuf, includes: Vec<String>, rpc_generator: pb_rs::types::RpcGeneratorFunction) -> Result<PathBuf, Error> {
	use pb_rs::types::Config;
	log::info!("Building protobuf for {:?}", &proto_filename);

	let out_dir = autogen_dir();
	std::fs::create_dir_all(&out_dir)?;
	let out_file = get_protobuf_generated_file(proto_filename);

    let config = Config {
        in_file: proto_filename.to_owned(),
        out_file: out_file.clone(), 
        single_module: true,
        import_search_path: vec![PathBuf::from("./schema")],
        no_output: false,
        error_cycle: false,
        headers: true,
		dont_use_cow: true,
        custom_struct_derive: vec!["derive_new::new".into()],
        custom_rpc_generator: rpc_generator,
		custom_includes: includes,
    };

    pb_rs::types::FileDescriptor::write_proto(&config).unwrap();

	log::info!("Pb-rs ran on {:?} and created {:?}", proto_filename, out_file);

	Ok(out_file)
}

/// Adds the file to IPFS so that 1) we can get it's hash and 2) So that we can generate a schema url from that hash
/// In parent program, lib.rs loads in the schema_link at compile time so that the library can use it.
pub fn add_file_to_ipfs(path: &PathBuf) -> Result<String, Error> {
	use hyper::rt::Future;
	use std::sync::{Arc, Mutex};
	let client = ipfs_api::IpfsClient::default();
	
	// Create atomics for hyper
	let should_panic = Arc::new(Mutex::new(false));
	let should_panic_clone = should_panic.clone();
	let hash = Arc::new(Mutex::new(String::new()));
	let hash_clone = hash.clone();

	log::info!("Adding {:?} to ipfs...", path);
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

	let hash = hash.lock().unwrap().clone();
    Ok(hash)
}

pub fn download_schema_from_ipfs(schema_name: &str, schema_ipfs_hash: &str) -> Result<PathBuf, Error> {
	log::debug!("Downloading schema {:?}...", schema_ipfs_hash);
	let schema_data = resolve_ipfs(schema_ipfs_hash)?;

	// TODO: Handle files of other protocols
	let schema_path = format!("./schema/{}.proto", schema_name);
	let schema_path = PathBuf::from(schema_path);
	log::debug!("Writing schema to {:?}", schema_path);
	write_to_file(&schema_path, std::str::from_utf8(&schema_data)?.to_string())?;
	log::debug!("...Finished downloading and saving schema.");
	Ok(schema_path)
}

fn resolve_ipfs(ipfs: &str) -> Result<Vec<u8>, Error> {
	let url = format!("http://localhost:8080/ipfs/{}", ipfs);
	log::trace!("Requesting {:?}...", url);
	let bytes = reqwest::get(&url)?.text()?.into_bytes();
	log::trace!("...Received {:?}", bytes);
	Ok(bytes)
}

pub fn remove_schema_urls_rs() {
	let schema_urls_rs = get_schema_urls_rs_path();
	if let Err(e) = std::fs::remove_file(&schema_urls_rs) {
		println!("Cannot remove {:?}. {:?}", schema_urls_rs, e);
	}
}

pub fn create_schema_urls_rs() -> Result<(), Error> {
	let schema_urls_rs = get_schema_urls_rs_path();

	let mut file_data = String::new();
	file_data += "// __SCHEMA_URLS__ Do not remove this line. This line is used to add new protocols.\n\n";

	file_data += "pub fn get_all_aliases() -> ::std::collections::HashMap<String, &'static str> {\n";
	file_data += "\tlet mut ret = ::std::collections::HashMap::new();\n";
	file_data += "\t// __SCHEMA_MAP_INSERT__ Do not remove this line. This line is used to add new protocols.\n";
	file_data += "\tret\n";
	file_data += "}\n";

	write_to_file(&schema_urls_rs, file_data)?;

	Ok(())
}

pub fn add_to_schema_urls_rs(base_name: &str, schema: &str) -> Result<(), Error> {
	let schema_urls_rs = get_schema_urls_rs_path();

	if !schema_urls_rs.exists() {
		create_schema_urls_rs()?;
	}

	// Add a line for direct lookup
	let line = format!("pub static SCHEMA_URL_{}: &str = \"{}\";\n", base_name.to_uppercase(), schema);
	let replace_line = "// __SCHEMA_URLS__";
	modify_file(&schema_urls_rs, replace_line, &(line + "\t" + replace_line))?;
	
	// Add a line for get_all_aliases
	let line = format!("ret.insert(\"{}\".to_string(), SCHEMA_URL_{});\n", base_name, base_name.to_uppercase());
	let replace_line = "// __SCHEMA_MAP_INSERT__";
	modify_file(&schema_urls_rs, replace_line, &(line + "\t" + replace_line))?;

	Ok(())
}

fn get_schema_urls_rs_path() -> PathBuf {
	let mut schema_urls_rs = autogen_dir();
	schema_urls_rs.push("schema_urls.rs");
	schema_urls_rs
}

pub fn file_missing_text(path: &PathBuf, text: &str) -> Result<bool, Error> {
	log::debug!("Loading file: {:?} and testing for {:?}", path, text);
	let file_data = ::std::fs::read_to_string(path)?;
	Ok(!file_data.contains(text))
}

pub fn append_to_file(path: &PathBuf, text: String) -> Result<(), Error> {
	let mut file_data = ::std::fs::read_to_string(path)?;
	file_data += &text;

	write_to_file(path, file_data)?;
	Ok(())
}

pub fn modify_file(path: &PathBuf, pretext: &str, posttext: &str) -> Result<(), Error> {
	let file_data = ::std::fs::read_to_string(path)?;
	let modified_file = file_data.replace(pretext, posttext);
	write_to_file(path, modified_file)?;
	Ok(())
}

pub fn for_all_in_dir(path_str: &str, func: fn(&PathBuf) -> Result<(), Error>) {
	use std::fs;
    let paths = fs::read_dir(path_str).unwrap();

    for path in paths {
		let path = path.unwrap().path();
		if let Err(e) = func(&path) {
			log::error!("{:?}", e);
		}
    }
}

pub fn write_to_file(new_file: &PathBuf, contents: String) -> Result<(), Error> {
	use std::io::Write;

	create_necessary_path_from_file_name(new_file)?;

	log::info!("Writing file: {:?}", new_file);
	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}

fn create_necessary_path_from_file_name(file_path: &PathBuf) -> Result<(), Error> {
	let mut just_the_path = file_path.clone();
	just_the_path.pop();
	std::fs::create_dir_all(just_the_path)?;
	Ok(())
}

fn get_protobuf_generated_file(proto_filename: &PathBuf) -> PathBuf {
	// Figure out the file that was generated.
	let base_name = base_name(proto_filename);
	let mut out_file = autogen_dir();
	out_file.push(format!("{}.rs", base_name));
	out_file
}

pub fn base_name(protobuf_path: &PathBuf) -> String {
	let base_name: String = protobuf_path.file_stem().unwrap().to_str().unwrap().to_string();
	base_name
}

fn autogen_dir() -> PathBuf {
	let mut dir = PathBuf::from(std::env::current_dir().unwrap());
	dir.push("src");
	dir.push("autogen");
	dir
}
