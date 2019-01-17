pub mod buildfunctions;

use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let generated_rs_file = buildfunctions::build_rust_code_from_protobuffer(proto_path)?;
	buildfunctions::modify_file(&generated_rs_file, "#![allow(clippy)]", "#![allow(clippy::all)]")?;

	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
	let _schema_url_path = buildfunctions::write_schema_url(proto_path, &hash)?;
	Ok(())
}

fn main()  {
	protocols::utils::initialize_standard_logging("")?;
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
}
