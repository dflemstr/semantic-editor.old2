//! Common RPC definitions for various communication protocols.
//!
//! These RPC definitions can be used for a wide variety of transport protocols as long as they can
//! agree on using protobuf-derived message schemata.
use bytes;
use failure;
use futures;
use prost_simple_rpc;
use slog;

use futures::prelude::{async, await};

use error;

pub mod http;
pub mod websocket;
pub mod ffi;

/// A generic RPC client connection.
#[derive(Clone, Debug)]
pub enum RpcClient<D> {
    /// The HTTP variant of this RPC client.
    Http(http::HttpRpcClient<D>),
    /// The WebSocket variant of this RPC client.
    WebSocket(websocket::WebSocketRpcClient<D>),
}

/// A generic RPC client call future.
#[derive(Debug)]
pub enum RpcClientCallFuture<D>
where
    D: prost_simple_rpc::descriptor::ServiceDescriptor + Clone + Send + 'static,
{
    /// The future is waiting on a HTTP RPC call.
    Http(<http::HttpRpcClient<D> as prost_simple_rpc::handler::Handler>::CallFuture),
    /// The future is waiting on a WebSocket RPC call.
    WebSocket(<websocket::WebSocketRpcClient<D> as prost_simple_rpc::handler::Handler>::CallFuture),
}

impl<D> RpcClient<D> {
    /// Create a new HTTP RPC instance.
    #[async]
    pub fn new(log: slog::Logger, base_url: String) -> error::Result<Self> {
        if base_url.starts_with("http://") {
            let inner = http::HttpRpcClient::new(log, base_url);
            Ok(RpcClient::Http(inner))
        } else if base_url.starts_with("ws://") {
            let inner = await!(websocket::WebSocketRpcClient::open(log, base_url))?;
            Ok(RpcClient::WebSocket(inner))
        } else {
            Err(failure::err_msg(format!(
                "Unsupported RPC transport scheme: {}",
                base_url
            )))
        }
    }
}

impl<D> prost_simple_rpc::handler::Handler for RpcClient<D>
where
    D: prost_simple_rpc::descriptor::ServiceDescriptor + Clone + Send + 'static,
{
    type Error = error::NestedError;
    type Descriptor = D;
    type CallFuture = RpcClientCallFuture<D>;

    fn call(&self, method: D::Method, input: bytes::Bytes) -> Self::CallFuture {
        match *self {
            RpcClient::Http(ref client) => RpcClientCallFuture::Http(client.call(method, input)),
            RpcClient::WebSocket(ref client) => RpcClientCallFuture::WebSocket(client.call(method, input)),
        }
    }
}

impl<D> futures::Future for RpcClientCallFuture<D>
where
    D: prost_simple_rpc::descriptor::ServiceDescriptor + Clone + Send + 'static,
{
    type Item = bytes::Bytes;
    type Error = error::NestedError;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match *self {
            RpcClientCallFuture::Http(ref mut future) => future.poll(),
            RpcClientCallFuture::WebSocket(ref mut future) => future.poll(),
        }
    }
}
