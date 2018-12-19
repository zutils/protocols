pub mod buildfunctions;

use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	let generated_rs_file = buildfunctions::build_rust_code_from_protobuffer(proto_path)?;
    buildfunctions::modify_file(&generated_rs_file, "#![allow(clippy)]", "#![allow(clippy::all)]")?;

    //let _generated_rs_file = buildfunctions::build_rust_rpc_code_from_protobuffer(proto_path)?;
	Ok(())
}

fn main()  {
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
}
