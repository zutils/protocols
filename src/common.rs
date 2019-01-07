use crate::{Destination, RpcData, VecModuleInfo, VecData, Data, VecRpcData, GenerateMessageInfo};
use failure::Error;

pub trait CommonModule {
    fn get_info(&self, p: &Destination) -> Result<VecModuleInfo, Error>;

    fn generate_message(&self, p: &GenerateMessageInfo) -> Result<Data, Error>;

    fn handle_raw(&self, p: &Data) -> Result<VecData, Error>;

    fn receive_rpc_as_client(&self, p: &RpcData) -> Result<VecRpcData, Error>;

    fn receive_rpc_as_server(&self, p: &RpcData) -> Result<VecRpcData, Error>;

    fn receive_public_rpc(&self, p: &RpcData) -> Result<VecRpcData, Error>;
}
