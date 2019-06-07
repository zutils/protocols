pub mod buildfunctions;
pub mod logging;

use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let _generated_rs_file = buildfunctions::build_rust_code_from_protobuffer_with_options(proto_path, Vec::new(),
		Box::new(|_, _| Ok(()) ))?;
	Ok(())
}

fn main() -> Result<(), Error>  {
	logging::initialize_standard_logging("")?;
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
	Ok(())
}
