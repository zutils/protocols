use crate::transport_autogen::transport::{Destination, RpcData, VecModuleInfo, VecData, Data, VecRpcData, GenerateMessageInfo};
use failure::Error;

pub trait CommonModule {
    fn get_info(&self, p: &Destination) -> Result<VecModuleInfo, Error>;

    fn generate_message(&self, p: &GenerateMessageInfo) -> Result<Data, Error>;

    fn handle_trusted(&self, p: &Data) -> Result<VecData, Error>;

    fn receive_trusted_rpc(&self, p: &RpcData) -> Result<VecRpcData, Error>;

    fn receive_untrusted_rpc(&self, p: &RpcData) -> Result<VecRpcData, Error>;
}
