use protocols::buildfunctions;
use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let _autogen_code = buildfunctions::build_rust_code_from_protobuffer(proto_path)?;
	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
    buildfunctions::add_to_schema_urls_rs(proto_path, &hash)?;
	Ok(())
}

fn main() -> Result<(), Error> {
	protocols::logging::initialize_standard_logging("")?;
	buildfunctions::remove_schema_urls_rs();
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
	Ok(())
}
