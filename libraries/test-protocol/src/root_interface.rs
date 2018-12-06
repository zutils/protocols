use root::Root;
use failure::{Error};
use protocols::pluginhandler::{MessageInfo, SubLibrary};

pub struct RootInterface;

impl SubLibrary for RootInterface {
    fn get_name(&self) -> String {
        return "Root".to_string();
    }

    fn handle(&self, info: MessageInfo) -> Result<Vec<MessageInfo>, Error> {
        let string: String = info.data.iter().map(|u: &u8| *u as char).collect();
        println!("Handling: {}", string);
        let structure: Root = serde_json::from_str(&string)?;
        println!("Received message: {:?}", structure);

        Ok(vec![create_submessage(structure, info)])
    }

    fn get_schema_url() -> String {
        return include_str!("../schema_urls/root.txt").to_string();
    }
}

/// Non-standard dynamic function.
#[no_mangle]
pub extern fn generate_root_message(schema_url: &str, data: &[u8]) -> Result<String, Error> {   
    let mut structure = Root::new();
    structure.set_schema_url(schema_url.to_string());
    structure.set_unencrypted_message(data.to_vec());
    Ok(serde_json::to_string(&structure)?)
}

fn create_submessage(root: Root, info: MessageInfo) -> MessageInfo {
    let schema_url = root.get_schema_url();
    let unencrypted_message = root.get_unencrypted_message();

    // Create history with 1 more element
    let mut history = info.history;
    history.push(schema_url.to_string());

    MessageInfo::new(history, schema_url, unencrypted_message)
}
