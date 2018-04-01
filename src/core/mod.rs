//! The core of the editor.
//!
//! Responsible for running the infrastructure used by all of the editor front-ends.
use std::net;

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

use tokio::prelude::*;

use error;
use schema::se::service as service_proto;
use schema::se::websocket as websocket_proto;
use version;

mod editor;
mod logger;
mod options;

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
    let server = service_proto::SemanticEditorServer::new(editor);

    let addr = "127.0.0.1:12345".parse()?;
    run_websocket_server(log, addr, server)?;
    Ok(())
}

fn run_websocket_server<H>(
    log: slog::Logger,
    addr: net::SocketAddr,
    handler: H,
) -> error::Result<()>
where
    H: prost_simple_rpc::handler::Handler,
{
    let log = log.new(o!("addr" => addr.to_string()));
    let listener = tokio::net::TcpListener::bind(&addr)?;

    let error_log = log.clone();
    let server = listener
        .incoming()
        .map_err(|e| failure::Error::from(e))
        .for_each(move |sock| handle_socket(log.clone(), handler.clone(), sock))
        .map_err(move |e| error!(error_log, "Error in websocket server: {}", e));

    tokio::run(server);
    Ok(())
}

fn handle_socket<H>(
    log: slog::Logger,
    handler: H,
    sock: tokio::net::TcpStream,
) -> Box<Future<Item = (), Error = error::Error> + Send>
where
    H: prost_simple_rpc::handler::Handler,
{
    Box::new(
        tokio_tungstenite::accept_async(sock)
            .map_err(|e| failure::Error::from(e))
            .map(move |ws_stream| handle_ws_stream(log, handler, ws_stream)),
    )
}

fn handle_ws_stream<H>(
    log: slog::Logger,
    handler: H,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) where
    H: prost_simple_rpc::handler::Handler,
{
    let (sink, stream) = ws_stream.split();
    let (tx, rx) = futures::sync::mpsc::unbounded();
    let ws_reader = stream
        .map_err(|e| failure::Error::from(e))
        .for_each(move |message| {
            use prost::Message;

            let tx = tx.clone();
            let handler = handler.clone();

            let data = message.into_data();
            // TODO(dflemstr): don't panic on errors here
            let request = websocket_proto::Request::decode::<bytes::Bytes>(data.into()).unwrap();
            let id = uuid::Uuid::from_bytes(&request.id).unwrap();
            debug!(log, "Received request";
            "request-id" => id.to_string());

            handle_request(handler, request)
                .map_err(|e| failure::Error::from(e))
                .and_then(move |r| {
                    debug!(log, "Sending response";
                    "request-id" => id.to_string());
                    let message = tungstenite::Message::binary(encode(r)?);
                    tx.unbounded_send(message)?;
                    Ok(())
                })
        });
    let ws_writer = rx.fold(sink, |mut sink, msg| {
        sink.start_send(msg).unwrap();
        Ok(sink)
    });
    let connection = ws_reader
        .map(|_| ())
        .map_err(|_| ())
        .select(ws_writer.map(|_| ()).map_err(|_| ()));
    tokio::spawn(connection.map(|_| ()).map_err(|_| ()));
}

fn handle_request<H>(
    mut handler: H,
    request: websocket_proto::Request,
) -> Box<Future<Item = websocket_proto::Response, Error = failure::Error> + Send>
where
    H: prost_simple_rpc::handler::Handler,
{
    use prost_simple_rpc::descriptor::MethodDescriptor;
    let service_name = <H::Descriptor as prost_simple_rpc::descriptor::ServiceDescriptor>::name();

    if request.service_name != service_name {
        return Box::new(futures::future::ok(websocket_proto::Response {
            id: request.id,
            data: vec![],
            error: websocket_proto::response::Error::ErrorServiceNotFound.into(),
        }));
    }

    let methods = <H::Descriptor as prost_simple_rpc::descriptor::ServiceDescriptor>::methods();

    if let Some(method_descriptor) = methods.iter().find(|m| request.method_name == m.name()) {
        let data = request.data;
        let id = request.id;
        Box::new(
            handler
                .call(*method_descriptor, data.into())
                .map(|r| websocket_proto::Response {
                    id: id,
                    data: r.to_vec(),
                    error: websocket_proto::response::Error::ErrorNone.into(),
                })
                .map_err(Into::into),
        )
    } else {
        Box::new(futures::future::ok(websocket_proto::Response {
            id: request.id,
            data: vec![],
            error: websocket_proto::response::Error::ErrorMethodNotFound.into(),
        }))
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
