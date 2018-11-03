extern crate protobuf_codegen_pure as pb;
extern crate protobuf_codegen;
extern crate ipfs_api;
extern crate hyper;
extern crate toml;
extern crate toml_query;
extern crate failure;

use self::failure::Error;
use std::fs::File;	

/// Build rust code from protobuffer. 
pub fn build_rust_code_from_protobuffer(proto_filename: &str) -> Result<(), Error> {
	let mut customize = pb::Customize::default();
	customize.serde_derive = Some(true);

	let args = pb::Args {
			out_dir: "src",
			input: &[proto_filename],
			includes: &[""],
			customize
	};

	pb::run(args).expect("protoc");

	Ok(())
}

/// Adds the file to IPFS so that 1) we can get it's hash and 2) So that we can generate a schema url from that hash
/// In parent program, lib.rs loads in the schema_url and description at compile time so that the library can use it.
pub fn add_file_and_write_ipfs_hash(proto_filename: &str) -> Result<(), Error> {
	use self::hyper::rt::Future;

	let client = ipfs_api::IpfsClient::default();
	let description: String = get_description_from_cargo_toml()?;
	
	println!("Adding file to ipfs...");
	let file = File::open(proto_filename)?;
	let req = client.add(file)
					.map(move |result| { 
						let schema_url = "https://ipfs.io/".to_string() + &result.hash;
                        println!("Writing {} to schema_url.txt", schema_url);
                        write_to_file("./schema_url.txt", &schema_url).unwrap();
						write_to_file("./description.txt", &description).unwrap();
                    })
					.map_err(|e| eprintln!("{}", e));

	hyper::rt::run(req);
    Ok(())
}

fn get_description_from_cargo_toml() -> Result<String, Error> {
	use self::toml::Value;
	use std::fs;
    use self::toml_query::read::TomlValueReadExt;

	let value = fs::read_to_string("Cargo.toml")?.parse::<Value>().unwrap();
	let value = match value.read("package.description").unwrap_or(None) {
        Some(ref v) => v.to_string(),
        None => "No Description".to_string(),
    };
    Ok(value)
}

fn write_to_file(new_file: &str, contents: &str) -> Result<(), Error> {
	use std::io::Write;

	let mut file = File::create(new_file)?;
	file.write_all(contents.as_bytes())?;
	Ok(())
}
