pub mod buildfunctions;
pub mod logging;

use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let _generated_rs_file = buildfunctions::build_rust_code_from_protobuffer_with_options(proto_path, Vec::new(),
		Box::new(|_, _| Ok(()) ))?;

	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
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
