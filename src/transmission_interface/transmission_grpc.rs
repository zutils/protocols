// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait CommonModule {
    fn get_info(&self, o: ::grpc::RequestOptions, p: super::transmission::Empty) -> ::grpc::SingleResponse<super::transmission::VecModuleInfo>;

    fn generate_default_message(&self, o: ::grpc::RequestOptions, p: super::transmission::GenerateMessageInfo) -> ::grpc::SingleResponse<super::transmission::Data>;

    fn handle_trusted(&self, o: ::grpc::RequestOptions, p: super::transmission::Data) -> ::grpc::SingleResponse<super::transmission::VecData>;

    fn receive_trusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::RpcData) -> ::grpc::SingleResponse<super::transmission::VecRpcData>;

    fn receive_untrusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::RpcData) -> ::grpc::SingleResponse<super::transmission::VecRpcData>;
}

// client

pub struct CommonModuleClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_get_info: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Empty, super::transmission::VecModuleInfo>>,
    method_generate_default_message: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::GenerateMessageInfo, super::transmission::Data>>,
    method_handle_trusted: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Data, super::transmission::VecData>>,
    method_receive_trusted_rpc: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::RpcData, super::transmission::VecRpcData>>,
    method_receive_untrusted_rpc: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::RpcData, super::transmission::VecRpcData>>,
}

impl ::grpc::ClientStub for CommonModuleClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        CommonModuleClient {
            grpc_client: grpc_client,
            method_get_info: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/CommonModule/get_info".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_generate_default_message: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/CommonModule/generate_default_message".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_handle_trusted: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/CommonModule/handle_trusted".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_receive_trusted_rpc: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/CommonModule/receive_trusted_rpc".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_receive_untrusted_rpc: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/CommonModule/receive_untrusted_rpc".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl CommonModule for CommonModuleClient {
    fn get_info(&self, o: ::grpc::RequestOptions, p: super::transmission::Empty) -> ::grpc::SingleResponse<super::transmission::VecModuleInfo> {
        self.grpc_client.call_unary(o, p, self.method_get_info.clone())
    }

    fn generate_default_message(&self, o: ::grpc::RequestOptions, p: super::transmission::GenerateMessageInfo) -> ::grpc::SingleResponse<super::transmission::Data> {
        self.grpc_client.call_unary(o, p, self.method_generate_default_message.clone())
    }

    fn handle_trusted(&self, o: ::grpc::RequestOptions, p: super::transmission::Data) -> ::grpc::SingleResponse<super::transmission::VecData> {
        self.grpc_client.call_unary(o, p, self.method_handle_trusted.clone())
    }

    fn receive_trusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::RpcData) -> ::grpc::SingleResponse<super::transmission::VecRpcData> {
        self.grpc_client.call_unary(o, p, self.method_receive_trusted_rpc.clone())
    }

    fn receive_untrusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::RpcData) -> ::grpc::SingleResponse<super::transmission::VecRpcData> {
        self.grpc_client.call_unary(o, p, self.method_receive_untrusted_rpc.clone())
    }
}

// server

pub struct CommonModuleServer;


impl CommonModuleServer {
    pub fn new_service_def<H : CommonModule + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/CommonModule",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/CommonModule/get_info".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/CommonModule/generate_default_message".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.generate_default_message(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/CommonModule/handle_trusted".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.handle_trusted(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/CommonModule/receive_trusted_rpc".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.receive_trusted_rpc(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/CommonModule/receive_untrusted_rpc".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.receive_untrusted_rpc(o, p))
                    },
                ),
            ],
        )
    }
}

// interface

pub trait TransportService {
    fn get_info(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission>;

    fn generate_default_message(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission>;

    fn handle_trusted(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission>;

    fn receive_trusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission>;

    fn receive_untrusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission>;
}

// client

pub struct TransportServiceClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_get_info: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Transmission, super::transmission::Transmission>>,
    method_generate_default_message: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Transmission, super::transmission::Transmission>>,
    method_handle_trusted: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Transmission, super::transmission::Transmission>>,
    method_receive_trusted_rpc: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Transmission, super::transmission::Transmission>>,
    method_receive_untrusted_rpc: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::transmission::Transmission, super::transmission::Transmission>>,
}

impl ::grpc::ClientStub for TransportServiceClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        TransportServiceClient {
            grpc_client: grpc_client,
            method_get_info: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/TransportService/get_info".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_generate_default_message: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/TransportService/generate_default_message".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_handle_trusted: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/TransportService/handle_trusted".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_receive_trusted_rpc: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/TransportService/receive_trusted_rpc".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_receive_untrusted_rpc: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/TransportService/receive_untrusted_rpc".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl TransportService for TransportServiceClient {
    fn get_info(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission> {
        self.grpc_client.call_unary(o, p, self.method_get_info.clone())
    }

    fn generate_default_message(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission> {
        self.grpc_client.call_unary(o, p, self.method_generate_default_message.clone())
    }

    fn handle_trusted(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission> {
        self.grpc_client.call_unary(o, p, self.method_handle_trusted.clone())
    }

    fn receive_trusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission> {
        self.grpc_client.call_unary(o, p, self.method_receive_trusted_rpc.clone())
    }

    fn receive_untrusted_rpc(&self, o: ::grpc::RequestOptions, p: super::transmission::Transmission) -> ::grpc::SingleResponse<super::transmission::Transmission> {
        self.grpc_client.call_unary(o, p, self.method_receive_untrusted_rpc.clone())
    }
}

// server

pub struct TransportServiceServer;


impl TransportServiceServer {
    pub fn new_service_def<H : TransportService + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/TransportService",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/TransportService/get_info".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.get_info(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/TransportService/generate_default_message".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.generate_default_message(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/TransportService/handle_trusted".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.handle_trusted(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/TransportService/receive_trusted_rpc".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.receive_trusted_rpc(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/TransportService/receive_untrusted_rpc".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.receive_untrusted_rpc(o, p))
                    },
                ),
            ],
        )
    }
}
