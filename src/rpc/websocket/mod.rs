//! An RPC implementation using WebSocket framing.

use bytes;
use futures;
use futures::sync::oneshot;
use prost;
use uuid;
use std::collections;
use std::sync;
use schema::se::websocket as websocket_proto;

pub mod ffi;

/// An RPC connection over a WebSocket.
#[derive(Debug)]
pub struct WebSocketRpc {
    web_socket: ffi::WebSocket,
    handler: WebSocketHandler,
}

/// A future that resolves when the underlying WebSocket is open.
#[derive(Debug)]
pub struct OpenFuture {
    open: oneshot::Receiver<()>,
    rpc: Option<WebSocketRpc>,
}

/// A future that resolves into the result of an RPC call.
#[derive(Debug)]
pub struct CallFuture(oneshot::Receiver<bytes::Bytes>);

/// An internal handler that handles incoming foreign raw WebSocket events.
#[derive(Clone, Debug)]
pub struct WebSocketHandler(sync::Arc<sync::Mutex<InnerWebSocketHandler>>);

#[derive(Debug)]
struct InnerWebSocketHandler {
    onopen: Option<oneshot::Sender<()>>,
    onmessage: collections::HashMap<uuid::Uuid, oneshot::Sender<bytes::Bytes>>,
}

impl WebSocketRpc {
    /// Opens a new RPC link over the specified WebSocket URL.
    ///
    /// Note that this RPC link will try to keep the connection established; there is a re-connect
    /// policy that is used if the connection drops.
    pub fn open(url: &str) -> OpenFuture {
        let web_socket = ffi::WebSocket::new(url);
        let (onopen, open) = oneshot::channel();
        let handler = WebSocketHandler(sync::Arc::new(sync::Mutex::new(InnerWebSocketHandler {
            onopen: Some(onopen),
            onmessage: collections::HashMap::new(),
        })));

        ffi::setWebSocketHandler(&web_socket, ffi::WebSocketHandler(handler.clone()));

        OpenFuture {
            open,
            rpc: Some(WebSocketRpc {
                web_socket,
                handler,
            }),
        }
    }
}

impl super::Client for WebSocketRpc {
    fn call(
        &mut self,
        service_name: &str,
        method_name: &str,
        input: bytes::Bytes,
    ) -> Box<futures::Future<Item = bytes::Bytes, Error = super::error::Error> + Send> {
        let id = uuid::Uuid::parse_str(&ffi::uuid::v4()).unwrap();

        let request = websocket_proto::Request {
            id: id.as_bytes().to_vec(),
            data: input.to_vec(),
            service_name: service_name.to_owned(),
            method_name: method_name.to_owned(),
        };
        debug!("Sending request: {:?}", request);

        let mut buffer = Vec::with_capacity(prost::Message::encoded_len(&request));
        // Should never fail since we are writing in-memory.
        prost::Message::encode(&request, &mut buffer).unwrap();

        let (onmessage, message) = oneshot::channel();
        self.handler
            .0
            .lock()
            .unwrap()
            .onmessage
            .insert(id, onmessage);
        self.web_socket.send(&buffer);
        Box::new(CallFuture(message))
    }
}

impl futures::Future for OpenFuture {
    type Item = WebSocketRpc;
    type Error = super::error::Error;

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
    type Error = super::error::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(futures::Async::Ready(data)) => Ok(futures::Async::Ready(data.into())),
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(ref err) => Err((*err).into()),
        }
    }
}

impl WebSocketHandler {
    /// Called when the associated WebSocket is closed.
    pub fn onclose(&self, code: u16, reason: &str, was_clean: bool) {
        info!("onclose({:?}, {:?}, {:?})", code, reason, was_clean);
    }

    /// Called when the associated WebSocket encountered an error.
    pub fn onerror(&self) {
        info!("onerror()");
    }

    /// Called when the associated WebSocket received new data.
    pub fn onmessage(&self, data: &[u8], origin: &str) {
        info!("Received data: {:?}", data);
        let response: websocket_proto::Response = match prost::Message::decode(data) {
            Err(e) => {
                error!("onmessage() failed to decode response: {}", e);
                return;
            }
            Ok(r) => r,
        };
        let uuid = match uuid::Uuid::from_bytes(&response.id) {
            Err(e) => {
                error!("onmessage() failed to decode response uuid: {}", e);
                return;
            }
            Ok(r) => r,
        };

        if let Some(onmessage) = self.0.lock().unwrap().onmessage.remove(&uuid) {
            if onmessage.send(response.data.into()).is_err() {
                error!("onmessage() failed to send oneshot notification");
            }
        } else {
            warn!("onmessage() called for unknown message id");
        }
    }

    /// Called when the associated WebSocket was successfully opened.
    pub fn onopen(&self) {
        if let Some(tx) = self.0.lock().unwrap().onopen.take() {
            match tx.send(()) {
                Err(()) => error!("onopen() failed to send oneshot notification"),
                Ok(()) => (),
            }
        } else {
            warn!("onopen() called for already-opened WebSocketHandler");
        }
    }
}
