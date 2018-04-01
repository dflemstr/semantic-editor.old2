//! The core of the editor.
//!
//! Responsible for running the infrastructure used by all of the editor front-ends.
#![allow(unused_qualifications)]

use std::mem;
use std::net;
use std::sync;

use bytes;
use failure;
use futures;
use slog;
use prost;
use prost_simple_rpc;
use tokio;
use tokio_tungstenite;
use tungstenite;
use uuid;

use futures::prelude::{async, await};

use error;
use schema::se::service as service_proto;
use schema::se::websocket as websocket_proto;
use version;

mod editor;
mod logger;
mod options;
#[cfg(feature = "standalone")]
mod standalone;

type Logger = slog::Logger<
    sync::Arc<slog::SendSyncRefUnwindSafeDrain<Ok = (), Err = slog::Never> + Send + Sync>,
>;

type RpcError = prost_simple_rpc::error::Error<error::NestedError>;

/// Runs the semantic editor core.
///
/// This can be treated like a main function; it will parse command line arguments etc.
pub fn run() -> error::Result<()> {
    use structopt::StructOpt;

    let options = options::Options::from_args();

    let log = logger::init(&options);

    version::init(&log);

    info!(log, "Parsed command-line options";
    "options" => format!("{:?}", options));

    let editor = editor::SemanticEditor::new(log.new(o!("component" => "editor")));
    let server_handler = service_proto::SemanticEditorServer::new(editor);

    let addr = "127.0.0.1:12345".parse()?;

    // Because the borrowchecker is tripping
    // TODO(dflemstr): there must be a way to avoid this.
    #[allow(unsafe_code)]
    let log: Logger = unsafe { mem::transmute(log) };

    run_server(log, addr, server_handler)?;
    Ok(())
}

fn run_server<H>(log: Logger, addr: net::SocketAddr, handler: H) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    use futures::Future;

    let addr_string = addr.to_string();
    let log = log.new(o!("addr" => addr_string));
    let err_log = log.clone();
    let listener = tokio::net::TcpListener::bind(&addr)?;
    let future = run_listener(log, listener, handler)
        .map_err(move |err| error!(err_log, "could not run server listener: {:?}", err));
    tokio::run(future);
    Ok(())
}

#[async]
fn run_listener<H>(log: Logger, listener: tokio::net::TcpListener, handler: H) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler<Error =RpcError>,
{
    #[async]
    for socket in listener.incoming() {
        let local_addr = socket.local_addr()?;
        let peer_addr = socket.peer_addr()?;
        let log = log.new(
            o!("local_addr" => local_addr.to_string(), "peer_addr" => peer_addr.to_string()),
        );
        await!(handle_socket(log, handler.clone(), socket))?;
    }

    Ok(())
}

#[async]
fn handle_socket<H>(log: Logger, handler: H, sock: tokio::net::TcpStream) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler<Error =RpcError>,
{
    let ws_stream = await!(tokio_tungstenite::accept_async(sock))?;
    match await!(handle_ws_stream(log.clone(), handler, ws_stream)) {
        Ok(()) => (),
        Err(err) => error!(log, "websocket stream terminated: {:?}", err),
    }
    Ok(())
}

#[async]
fn handle_ws_stream<H>(
    log: Logger,
    handler: H,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler<Error =RpcError>,
{
    use futures::Future;
    use futures::Stream;

    let (sink, stream) = ws_stream.split();
    let (tx, rx) = futures::sync::mpsc::unbounded();

    let ws_reader = run_ws_reader(log, handler, stream, tx);
    let ws_writer = run_ws_writer(sink, rx);

    await!(ws_reader.select(ws_writer).map_err(|(e, _)| e))?;

    Ok(())
}

#[async]
fn run_ws_reader<H, S>(
    log: Logger,
    handler: H,
    stream: S,
    response_tx: futures::sync::mpsc::UnboundedSender<tungstenite::Message>,
) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler<Error =RpcError>,
    S: futures::Stream<Item = tungstenite::Message, Error = tungstenite::Error> + 'static,
{
    use prost::Message;

    #[async]
    for message in stream {
        let handler = handler.clone();
        let data = message.into_data();
        let request = websocket_proto::Request::decode::<bytes::Bytes>(data.into())?;
        let id = uuid::Uuid::from_bytes(&request.id)?;
        debug!(log, "Received request";
        "request-id" => id.to_string());

        let response = await!(handle_ws_request(handler, request))?;

        debug!(log, "Sending response";
        "request-id" => id.to_string());

        let message = tungstenite::Message::binary(encode(response)?);

        response_tx.unbounded_send(message)?;
    }

    Ok(())
}

#[async]
fn run_ws_writer<S>(mut sink: S,
                    response_rx: futures::sync::mpsc::UnboundedReceiver<tungstenite::Message>)
-> error::Result<()>
    where S: futures::Sink<SinkItem = tungstenite::Message, SinkError = tungstenite::Error> + 'static {
    use futures::Stream;

    #[async]
    for msg in response_rx.map_err(|_| failure::err_msg("broken response pipe")) {
        sink.start_send(msg)?;
    }

    Ok(())
}

#[async]
fn handle_ws_request<H>(
    handler: H,
    request: websocket_proto::Request,
) -> error::Result<websocket_proto::Response>
where
    H: prost_simple_rpc::handler::Handler<Error =RpcError>,
{
    use prost_simple_rpc::descriptor::MethodDescriptor;
    use prost_simple_rpc::descriptor::ServiceDescriptor;

    let service_name = H::Descriptor::name();
    let methods = H::Descriptor::methods();

    if request.service_name != service_name {
        return Ok(error_response(
            request.id,
            websocket_proto::response::Error::ErrorServiceNotFound,
        ));
    }

    if let Some(method_descriptor) = methods.iter().find(|m| request.method_name == m.name()) {
        let method_descriptor = method_descriptor.clone();
        let response = await!(handler.call(method_descriptor, request.data.into()))?;
        Ok(data_response(request.id, response.to_vec()))
    } else {
        Ok(error_response(
            request.id,
            websocket_proto::response::Error::ErrorMethodNotFound,
        ))
    }
}

fn data_response(request_id: Vec<u8>, data: Vec<u8>) -> websocket_proto::Response {
    websocket_proto::Response {
        id: request_id,
        data,
        error: websocket_proto::response::Error::ErrorNone.into(),
    }
}

fn error_response(
    request_id: Vec<u8>,
    error: websocket_proto::response::Error,
) -> websocket_proto::Response {
    websocket_proto::Response {
        id: request_id,
        data: vec![],
        error: error.into(),
    }
}

fn encode<M>(message: M) -> error::Result<Vec<u8>>
where
    M: prost::Message,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.to_vec())
}
