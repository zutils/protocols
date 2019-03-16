pub mod buildfunctions;
pub mod logging;

use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let includes = vec!["//__SCHEMA_URL__".to_string()];
	let generated_rs_file = buildfunctions::build_rust_code_from_protobuffer_with_options(proto_path, includes,
		Box::new(|_, _| Ok(()) ))?;

	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
	buildfunctions::replace_schema_url_comment_with_hash_constant(&generated_rs_file, &hash)?;
	let base_name = buildfunctions::base_name(proto_path);
	buildfunctions::add_to_schema_urls_rs(&base_name, &hash)?;
	Ok(())
}

fn main() -> Result<(), Error>  {
	logging::initialize_standard_logging("")?;
	buildfunctions::remove_schema_urls_rs();
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
	Ok(())
}
