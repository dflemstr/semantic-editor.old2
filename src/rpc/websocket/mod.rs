//! An RPC implementation using WebSocket framing.

use bytes;
use error;
use failure;
use futures;
use futures::sync::oneshot;
use prost;
use prost_simple_rpc;
use schema::se::transport as transport_proto;
use slog;
use std::collections;
use std::marker;
use std::sync;
use uuid;

pub mod ffi;

/// An RPC connection over a WebSocket.
#[derive(Clone, Debug)]
pub struct WebSocketRpcClient<D> {
    log: slog::Logger,
    inner: sync::Arc<sync::Mutex<Inner>>,
    _descriptor: marker::PhantomData<D>,
}

#[derive(Debug)]
struct Inner {
    web_socket: ffi::WebSocket,
    handler: WebSocketHandler,
}

/// A future that resolves when the underlying WebSocket is open.
#[derive(Debug)]
pub struct OpenFuture<D> {
    open: oneshot::Receiver<()>,
    rpc: Option<WebSocketRpcClient<D>>,
}

/// A future that resolves into the result of an RPC call.
#[derive(Debug)]
pub struct CallFuture(oneshot::Receiver<bytes::Bytes>);

/// An internal handler that handles incoming foreign raw WebSocket events.
#[derive(Clone, Debug)]
pub struct WebSocketHandler {
    log: slog::Logger,
    inner: sync::Arc<sync::Mutex<InnerWebSocketHandler>>,
}

#[derive(Debug)]
struct InnerWebSocketHandler {
    onopen: Option<oneshot::Sender<()>>,
    onmessage: collections::HashMap<uuid::Uuid, oneshot::Sender<bytes::Bytes>>,
}

impl<D> WebSocketRpcClient<D> {
    /// Opens a new RPC link over the specified WebSocket URL.
    ///
    /// Note that this RPC link will try to keep the connection established; there is a re-connect
    /// policy that is used if the connection drops.
    pub fn open(log: slog::Logger, url: String) -> OpenFuture<D> {
        let web_socket = ffi::WebSocket::new(&url);
        let (onopen, open) = oneshot::channel();

        let inner = sync::Arc::new(sync::Mutex::new(InnerWebSocketHandler {
            onopen: Some(onopen),
            onmessage: collections::HashMap::new(),
        }));

        let handler_log = log.new(o!("component" => "web-socket-handler"));
        let handler = WebSocketHandler {
            log: handler_log,
            inner,
        };

        ffi::setWebSocketHandler(&web_socket, ffi::WebSocketHandler(handler.clone()));

        let inner = sync::Arc::new(sync::Mutex::new(Inner {
            web_socket,
            handler,
        }));

        let log = log.new(o!("component" => "web-socket-rpc"));
        let rpc = Some(WebSocketRpcClient {
            log,
            inner,
            _descriptor: marker::PhantomData,
        });

        OpenFuture { open, rpc }
    }
}

impl<D> prost_simple_rpc::handler::Handler for WebSocketRpcClient<D>
where
    D: prost_simple_rpc::descriptor::ServiceDescriptor + Clone + Send + 'static,
{
    type Error = error::NestedError;
    type Descriptor = D;
    type CallFuture = CallFuture;

    fn call(&self, method: D::Method, input: bytes::Bytes) -> Self::CallFuture {
        use prost_simple_rpc::descriptor::MethodDescriptor;

        let id = uuid::Uuid::parse_str(&super::ffi::uuid::v4()).unwrap();
        let log = self.log.new(o!("request-id" => id.to_string()));

        let request = transport_proto::Request {
            id: id.as_bytes().to_vec(),
            data: input.to_vec(),
            service_name: format!("{}.{}", D::package(), D::proto_name()),
            method_name: method.proto_name().to_owned(),
        };
        debug!(log, "Sending request";
        "request" => format!("{:?}", request));

        let mut buffer = Vec::with_capacity(prost::Message::encoded_len(&request));
        // Should never fail since we are writing in-memory.
        prost::Message::encode(&request, &mut buffer).unwrap();

        let (onmessage, message) = oneshot::channel();

        let inner = self.inner.lock().unwrap();
        let mut inner_handler = inner.handler.inner.lock().unwrap();
        inner_handler.onmessage.insert(id, onmessage);
        inner.web_socket.send(&buffer);

        CallFuture(message)
    }
}

impl<D> futures::Future for OpenFuture<D> {
    type Item = WebSocketRpcClient<D>;
    type Error = error::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.open.poll() {
            Ok(futures::Async::Ready(())) => Ok(futures::Async::Ready(self.rpc.take().unwrap())),
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(ref err) => Err((*err).into()),
        }
    }
}

impl futures::Future for CallFuture {
    type Item = bytes::Bytes;
    type Error = error::NestedError;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(futures::Async::Ready(data)) => Ok(futures::Async::Ready(data.into())),
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(ref err) => Err(error::nested_error(failure::Error::from(*err))),
        }
    }
}

impl WebSocketHandler {
    /// Called when the associated WebSocket is closed.
    pub fn onclose(&self, _code: u16, _reason: &str, _was_clean: bool) {}

    /// Called when the associated WebSocket encountered an error.
    pub fn onerror(&self) {}

    /// Called when the associated WebSocket received new data.
    pub fn onmessage(&self, data: &[u8], _origin: &str) {
        let log = self.log.new(o!("method" => "onmessage"));
        let response: transport_proto::Response = match prost::Message::decode(data) {
            Err(e) => {
                error!(log, "Failed to decode response";
                "error" => format!("{}", e));
                return;
            }
            Ok(r) => r,
        };

        let id = match uuid::Uuid::from_bytes(&response.id) {
            Err(e) => {
                error!(log, "Failed to decode response uuid";
                "error" => format!("{}", e));
                return;
            }
            Ok(r) => r,
        };
        let log = log.new(o!("request-id" => id.to_string()));
        debug!(log, "Received response";
        "response" => format!("{:?}", response));

        if let Some(onmessage) = self.inner.lock().unwrap().onmessage.remove(&id) {
            if onmessage.send(response.data.into()).is_err() {
                error!(log, "Failed to send oneshot notification");
            }
        } else {
            warn!(log, "Called for unknown message id");
        }
    }

    /// Called when the associated WebSocket was successfully opened.
    pub fn onopen(&self) {
        let log = self.log.new(o!("method" => "onopen"));
        if let Some(tx) = self.inner.lock().unwrap().onopen.take() {
            match tx.send(()) {
                Err(()) => error!(log, "Failed to send oneshot notification"),
                Ok(()) => (),
            }
        } else {
            warn!(log, "Called for already-opened WebSocketHandler");
        }
    }
}
