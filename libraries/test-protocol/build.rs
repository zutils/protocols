use protocols::buildfunctions;
use failure::Error;

fn main() -> Result<(), Error> {
	protocols::logging::initialize_standard_logging("")?;
	buildfunctions::remove_schema_urls_rs();
	buildfunctions::for_all_in_dir("./schema/", |path| {
		println!("Generating code for {:?}", path);
		buildfunctions::hash_protobuf_and_generate_code(path)
	});
	Ok(())
}

/*use std::path::PathBuf;

pub fn hash_protobuf_and_generate_code(proto_path: &PathBuf) -> Result<(), Error> {
	let includes = vec![
		"use protocols::{Data, RpcData};".to_string(),
		"//__SCHEMA_URL__".to_string()
	];

	let generated_rs_file = buildfunctions::build_rust_code_from_protobuffer_with_options(proto_path, includes,
		Box::new(|_, _| Ok(())))?;

	//let generated_rs_file = build_rust_code_from_protobuffer(proto_path)?;
	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
	buildfunctions::replace_schema_url_comment_with_hash_constant(&generated_rs_file, &hash)?;
	buildfunctions::add_to_schema_urls_rs(&buildfunctions::base_name(proto_path), &hash)?;
	Ok(())
}

fn main() -> Result<(), Error> {
	protocols::logging::initialize_standard_logging("")?;
	buildfunctions::remove_schema_urls_rs();
	buildfunctions::for_all_in_dir("./schema/", |path| hash_protobuf_and_generate_code(path));
	Ok(())
}*/
