//! An RPC implementation using normal HTTP requests.
use std::marker;
use std::sync;

use bytes;
use failure;
use futures;
use futures::sync::oneshot;
use prost;
use prost_simple_rpc;
use slog;
use uuid;

use error;
use schema::se::transport as transport_proto;

pub mod ffi;

/// An RPC connection over HTTP.
#[derive(Clone, Debug)]
pub struct HttpRpcClient<D> {
    log: slog::Logger,
    base_url: String,
    _descriptor: marker::PhantomData<D>,
}

/// An internal handler that handles fetch results.
#[derive(Debug)]
pub struct HttpFetchHandler {
    log: slog::Logger,
    tx: sync::Mutex<Option<oneshot::Sender<Result<bytes::Bytes, error::NestedError>>>>,
}

/// A future that resolves into the result of an RPC call.
#[derive(Debug)]
pub struct CallFuture {
    log: slog::Logger,
    id: uuid::Uuid,
    rx: oneshot::Receiver<Result<bytes::Bytes, error::NestedError>>,
}

impl<D> HttpRpcClient<D> {
    /// Create a new HTTP RPC instance.
    pub fn new(log: slog::Logger, base_url: String) -> Self {
        HttpRpcClient {
            log,
            base_url,
            _descriptor: marker::PhantomData,
        }
    }
}

impl<D> prost_simple_rpc::handler::Handler for HttpRpcClient<D>
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

        let url = format!(
            "{}/{}/{}",
            self.base_url, request.service_name, request.method_name
        );

        let (tx, rx) = oneshot::channel();
        let tx = sync::Mutex::new(Some(tx));
        let ret_log = log.clone();
        ffi::performFetch(
            &url,
            &buffer,
            ffi::HttpFetchHandler(HttpFetchHandler { log, tx }),
        );

        CallFuture {
            log: ret_log,
            id,
            rx,
        }
    }
}

impl HttpFetchHandler {
    fn resolve(&self, data: &[u8]) {
        if let Some(tx) = self.take_tx() {
            let data = bytes::Bytes::from(data);
            trace!(self.log, "resolved: {:?}", data);
            tx.send(Ok(data)).unwrap();
        }
    }

    fn reject(&self, error: &str) {
        if let Some(tx) = self.take_tx() {
            trace!(self.log, "rejected: {:?}", error);
            tx.send(Err(error::nested_error(failure::err_msg(error.to_owned()))))
                .unwrap();
        }
    }

    fn take_tx(&self) -> Option<oneshot::Sender<Result<bytes::Bytes, error::NestedError>>> {
        let mut maybe_tx = match self.tx.lock() {
            Ok(s) => s,
            Err(error) => {
                error!(
                    self.log,
                    "deadlock when trying to take HTTP fetch handler tx: {}", error
                );
                return None;
            }
        };

        match maybe_tx.take() {
            Some(tx) => Some(tx),
            None => {
                error!(self.log, "HTTP fetch handler called twice");
                None
            }
        }
    }
}

impl futures::Future for CallFuture {
    type Item = bytes::Bytes;
    type Error = error::NestedError;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        use prost::Message;

        match self.rx.poll() {
            Ok(futures::Async::Ready(Ok(data))) => {
                trace!(self.log, "ready: {:?}", data);
                let response = transport_proto::Response::decode(data)
                    .map_err(|e| error::nested_error(error::Error::from(e)))?;

                let id = uuid::Uuid::from_bytes(&response.id).unwrap();

                if id != self.id {
                    Err(error::nested_error(failure::err_msg(format!(
                        "request/response id mismatch: {}/{}",
                        self.id, id
                    ))))
                } else {
                    Ok(futures::Async::Ready(response.data.into()))
                }
            }
            Ok(futures::Async::Ready(Err(err))) => Err(err),
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(err) => Err(error::nested_error(error::Error::from(err))),
        }
    }
}
