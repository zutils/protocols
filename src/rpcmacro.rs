#[macro_export]
macro_rules! implCommonModule {
    ( $moduleName:ident , $schema:expr ) => {
        struct ClientRPCHandler;
        struct ServerRPCHandler;
        struct PublicRPCHandler;
        impl $moduleName::ClientRPC for ClientRPCHandler;
        impl $moduleName::ServerRPC for ServerRPCHandler;
        impl $moduleName::PublicRPC for PublicRPCHandler;

        pub struct Interface;
        impl CommonModule for Interface {
            fn get_info(&self, _: &Destination) -> Result<VecModuleInfo, failure::Error> {
                let info = ModuleInfo::new($schema.into(), stringify!($moduleName).to_string());
                Ok(VecModuleInfo::new(vec![info]))
            }

            fn receive_rpc_as_client(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
                $moduleName::handle_ClientRPC(data, ClientRPCHandler{})
            }

            fn receive_rpc_as_server(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
                $moduleName::handle_ServerRPC(data, ServerRPCHandler{})
            }

            fn receive_public_rpc(&self, data: &RpcData) -> Result<VecRpcData, failure::Error> {
                $moduleName::handle_PublicRPC(data, PublicRPCHandler{})
            }
        }
    }
}