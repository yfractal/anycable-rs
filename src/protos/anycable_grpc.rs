// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

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

const METHOD_RPC_CONNECT: ::grpcio::Method<super::anycable::ConnectionRequest, super::anycable::ConnectionResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/anycable.RPC/Connect",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_RPC_COMMAND: ::grpcio::Method<super::anycable::CommandMessage, super::anycable::CommandResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/anycable.RPC/Command",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_RPC_DISCONNECT: ::grpcio::Method<super::anycable::DisconnectRequest, super::anycable::DisconnectResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/anycable.RPC/Disconnect",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct RpcClient {
    client: ::grpcio::Client,
}

impl RpcClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        RpcClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn connect_opt(&self, req: &super::anycable::ConnectionRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::anycable::ConnectionResponse> {
        self.client.unary_call(&METHOD_RPC_CONNECT, req, opt)
    }

    pub fn connect(&self, req: &super::anycable::ConnectionRequest) -> ::grpcio::Result<super::anycable::ConnectionResponse> {
        self.connect_opt(req, ::grpcio::CallOption::default())
    }

    pub fn connect_async_opt(&self, req: &super::anycable::ConnectionRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::ConnectionResponse>> {
        self.client.unary_call_async(&METHOD_RPC_CONNECT, req, opt)
    }

    pub fn connect_async(&self, req: &super::anycable::ConnectionRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::ConnectionResponse>> {
        self.connect_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn command_opt(&self, req: &super::anycable::CommandMessage, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::anycable::CommandResponse> {
        self.client.unary_call(&METHOD_RPC_COMMAND, req, opt)
    }

    pub fn command(&self, req: &super::anycable::CommandMessage) -> ::grpcio::Result<super::anycable::CommandResponse> {
        self.command_opt(req, ::grpcio::CallOption::default())
    }

    pub fn command_async_opt(&self, req: &super::anycable::CommandMessage, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::CommandResponse>> {
        self.client.unary_call_async(&METHOD_RPC_COMMAND, req, opt)
    }

    pub fn command_async(&self, req: &super::anycable::CommandMessage) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::CommandResponse>> {
        self.command_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn disconnect_opt(&self, req: &super::anycable::DisconnectRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::anycable::DisconnectResponse> {
        self.client.unary_call(&METHOD_RPC_DISCONNECT, req, opt)
    }

    pub fn disconnect(&self, req: &super::anycable::DisconnectRequest) -> ::grpcio::Result<super::anycable::DisconnectResponse> {
        self.disconnect_opt(req, ::grpcio::CallOption::default())
    }

    pub fn disconnect_async_opt(&self, req: &super::anycable::DisconnectRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::DisconnectResponse>> {
        self.client.unary_call_async(&METHOD_RPC_DISCONNECT, req, opt)
    }

    pub fn disconnect_async(&self, req: &super::anycable::DisconnectRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::anycable::DisconnectResponse>> {
        self.disconnect_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Rpc {
    fn connect(&mut self, ctx: ::grpcio::RpcContext, req: super::anycable::ConnectionRequest, sink: ::grpcio::UnarySink<super::anycable::ConnectionResponse>);
    fn command(&mut self, ctx: ::grpcio::RpcContext, req: super::anycable::CommandMessage, sink: ::grpcio::UnarySink<super::anycable::CommandResponse>);
    fn disconnect(&mut self, ctx: ::grpcio::RpcContext, req: super::anycable::DisconnectRequest, sink: ::grpcio::UnarySink<super::anycable::DisconnectResponse>);
}

pub fn create_rpc<S: Rpc + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_RPC_CONNECT, move |ctx, req, resp| {
        instance.connect(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_RPC_COMMAND, move |ctx, req, resp| {
        instance.command(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_unary_handler(&METHOD_RPC_DISCONNECT, move |ctx, req, resp| {
        instance.disconnect(ctx, req, resp)
    });
    builder.build()
}
