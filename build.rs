use protocols::buildfunctions;
use failure::Error;
use std::path::PathBuf;

fn create_protobuf(proto_path: &PathBuf) -> Result<(), Error> {
	//let generated_rs_file = 
    buildfunctions::build_rust_code_from_protobuffer(proto_path)?;

    // This modification is necessary because we need to be able to own the data.
    //modify_file(&generated_rs_file, "r.read_string(bytes).map(Cow::Borrowed)?", "Cow::Owned(r.read_string(bytes)?.to_string())")?;
    //modify_file(&generated_rs_file, "r.read_bytes(bytes).map(Cow::Borrowed)?", "Cow::Owned(r.read_bytes(bytes)?.to_vec())")?;
	
    //buildfunctions::build_rust_rpc_code_from_protobuffer(proto_path)?;
	Ok(())
}

fn main()  {
	buildfunctions::for_all_in_dir("./schema/", |path| create_protobuf(path));
}
