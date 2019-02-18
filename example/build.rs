use protocols::buildfunctions;
use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
    log::info!("Building rust code: {:?}", proto_path);
	let _autogen_code = buildfunctions::build_rust_code_from_protobuffer(proto_path)?;
	Ok(())
}

fn main() -> Result<(), Error>  {
    protocols::logging::initialize_standard_logging("")?;

    let schema = include_str!("../libraries/test-protocol/schema_urls/test.txt");
    let _proto_path = buildfunctions::download_schema_from_ipfs("test", schema);

	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));

    Ok(())
}
