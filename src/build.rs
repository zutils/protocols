pub mod buildfunctions;
pub mod logging;

use failure::Error;
use std::path::PathBuf;
use std::io::Write;

fn autogenerate_rpc<W: Write + ?Sized>(rpc: &pb_rs::types::RpcService, w: &mut W) -> Result<(), pb_rs::errors::Error> {

	Ok(())
}

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let _generated_rs_file = buildfunctions::build_rust_code_from_protobuffer_with_options(proto_path, Vec::new(),
		Box::new(|rpc, writer| autogenerate_rpc(rpc, writer)))?;

	let hash = buildfunctions::add_file_to_ipfs(proto_path)?;
	let _schema_url_path = buildfunctions::write_schema_url(proto_path, &hash)?;
	Ok(())
}

fn main() -> Result<(), Error>  {
	logging::initialize_standard_logging("")?;
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
	Ok(())
}
