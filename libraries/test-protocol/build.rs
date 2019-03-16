use protocols::buildfunctions;
use failure::Error;
use std::path::PathBuf;

fn main() -> Result<(), Error> {
	protocols::logging::initialize_standard_logging("")?;
	buildfunctions::remove_schema_urls_rs();
	buildfunctions::for_all_in_dir("./schema/", |path| buildfunctions::hash_protobuf_and_generate_code(path));
	Ok(())
}
