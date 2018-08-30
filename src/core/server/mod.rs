use bytes;
use failure;
use futures;
use hyper;
use prost;
use prost_simple_rpc;
use slog;

use futures::prelude::{async, await};

use error;
use schema::se::transport as transport_proto;

mod standalone;

type RpcError = prost_simple_rpc::error::Error<error::NestedError>;

const REQUEST_CONTENT_TYPE: &str = "application/x-semantic-editor-request";
const RESPONSE_CONTENT_TYPE: &str = "application/x-semantic-editor-response";

pub struct Server<H> {
    log: slog::Logger,
    handler: H,
}

struct HandlerHyperService<H> {
    handler: H,
}

#[derive(Debug)]
struct Request {
    id: Vec<u8>,
    data: Vec<u8>,
    service_name: String,
    method_name: String,
}

impl<H> Server<H>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError> + Sync,
{
    pub fn new(log: slog::Logger, handler: H) -> Self {
        Server { log, handler }
    }

    pub fn run(self) -> error::Result<()> {
        use futures::Future;

        let addr = "127.0.0.1:12345".parse()?;

        let server_handler = self.handler.clone();
        let server = hyper::Server::bind(&addr).serve(move || {
            let handler = server_handler.clone();
            HandlerHyperService { handler }
        });

        let error_log = self.log.clone();
        hyper::rt::run(server.map_err(move |e| error!(error_log, "server error: {}", e)));
        Ok(())
    }
}

impl<H> hyper::service::Service for HandlerHyperService<H>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    type ReqBody = hyper::Body;
    type ResBody = hyper::Body;
    type Error = failure::Compat<error::Error>;
    type Future =
        Box<futures::Future<Item = hyper::Response<Self::ResBody>, Error = Self::Error> + Send>;

    fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
        use futures::Future;
        Box::new(call_raw(self.handler.clone(), req).map_err(|e| failure::Error::compat(e)))
    }
}

impl<H> futures::IntoFuture for HandlerHyperService<H> {
    type Future = futures::future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = failure::Compat<error::Error>;

    fn into_future(self) -> Self::Future {
        futures::future::ok(self)
    }
}

#[async]
fn call_raw<H>(
    handler: H,
    req: hyper::Request<hyper::Body>,
) -> error::Result<hyper::Response<hyper::Body>>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    if req.method() == &hyper::Method::OPTIONS {
        let mut response = hyper::Response::new(hyper::Body::empty());
        *response.status_mut() = hyper::StatusCode::OK;
        Ok(with_cors_headers(response))
    } else if is_static_file_request(&req) {
        Ok(with_cors_headers(serve_static_file(&req)))
    } else if let Some(request) = await!(parse_rpc_request(req))? {
        await!(handle_request(handler, request))
    } else {
        let mut response = hyper::Response::new(hyper::Body::empty());
        *response.status_mut() = hyper::StatusCode::NOT_FOUND;
        Ok(response)
    }
}

#[async]
fn handle_request<H>(handler: H, request: Request) -> error::Result<hyper::Response<hyper::Body>>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    use prost_simple_rpc::descriptor::MethodDescriptor;
    use prost_simple_rpc::descriptor::ServiceDescriptor;

    let service_name = format!(
        "{}.{}",
        H::Descriptor::package(),
        H::Descriptor::proto_name()
    );
    let methods = H::Descriptor::methods();

    if request.service_name != service_name {
        Ok(hyper_response(error_code_response(
            request.id,
            transport_proto::response::ErrorCode::ServiceNotFound,
        )))
    } else if let Some(method_descriptor) = methods
        .iter()
        .find(|m| request.method_name == m.proto_name())
    {
        let method_descriptor = method_descriptor.clone();
        match await!(handler.call(method_descriptor, request.data.into())) {
            Ok(response_data) => Ok(hyper_response(data_response(
                request.id,
                response_data.to_vec(),
            ))),
            Err(err) => Ok(hyper_response(error_response(request.id, &err))),
        }
    } else {
        Ok(hyper_response(error_code_response(
            request.id,
            transport_proto::response::ErrorCode::MethodNotFound,
        )))
    }
}

#[async]
fn parse_rpc_request(req: hyper::Request<hyper::Body>) -> error::Result<Option<Request>> {
    use futures::Stream;
    use prost::Message;

    if req.headers().get(hyper::header::CONTENT_TYPE)
        != Some(&hyper::header::HeaderValue::from_static(
            REQUEST_CONTENT_TYPE,
        )) {
        return Ok(None);
    }

    let (service_name, method_name) = {
        let parts = req.uri().path().split("/").collect::<Vec<_>>();
        if parts.len() == 3 && parts[0] == "" {
            (parts[1].to_owned(), parts[2].to_owned())
        } else {
            return Ok(None);
        }
    };

    let body = bytes::Bytes::from(await!(req.into_body().concat2())?);
    if let Some(request) = transport_proto::Request::decode(body).ok() {
        let id = request.id;
        let data = request.data;
        Ok(Some(Request {
            id,
            data,
            service_name,
            method_name,
        }))
    } else {
        Ok(None)
    }
}

fn is_static_file_request(request: &hyper::Request<hyper::Body>) -> bool {
    standalone::file_exists(canonicalize_path(request.uri().path()))
}

fn serve_static_file(request: &hyper::Request<hyper::Body>) -> hyper::Response<hyper::Body> {
    let path = canonicalize_path(request.uri().path());

    if request
        .headers()
        .get(hyper::header::ACCEPT_ENCODING)
        .map(|h| h.to_str().map(|s| s.contains("br")).unwrap_or(false))
        .unwrap_or(false)
    {
        let data = standalone::brotli_compressed_file(path).unwrap();
        let mut response = hyper::Response::new(hyper::Body::wrap_stream(data.contents));
        *response.status_mut() = hyper::StatusCode::OK;
        response.headers_mut().insert(
            hyper::header::CONTENT_ENCODING,
            hyper::header::HeaderValue::from_static("br"),
        );
        response
    } else {
        let data = standalone::file(path).unwrap();
        let mut response = hyper::Response::new(hyper::Body::wrap_stream(data.contents));
        *response.status_mut() = hyper::StatusCode::OK;
        response
            .headers_mut()
            .insert(hyper::header::CONTENT_LENGTH, data.size.into());
        response
    }
}

fn canonicalize_path(path: &str) -> &str {
    if path.is_empty() || path == "/" {
        "index.html"
    } else {
        &path[1..]
    }
}

fn hyper_response(response: transport_proto::Response) -> hyper::Response<hyper::Body> {
    let data = encode(response).unwrap();
    let len = data.len();
    let mut response = hyper::Response::new(data.into());
    *response.status_mut() = hyper::StatusCode::OK;
    response.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static(RESPONSE_CONTENT_TYPE),
    );
    response
        .headers_mut()
        .insert(hyper::header::CONTENT_LENGTH, len.into());

    with_cors_headers(response)
}

fn with_cors_headers<B>(mut response: hyper::Response<B>) -> hyper::Response<B> {
    response.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        hyper::header::HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
        hyper::header::HeaderValue::from_static("POST"),
    );
    response.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
        hyper::header::HeaderValue::from_static("*"),
    );
    response
}

fn data_response(request_id: Vec<u8>, data: Vec<u8>) -> transport_proto::Response {
    transport_proto::Response {
        id: request_id,
        data,
        error_code: transport_proto::response::ErrorCode::None.into(),
        ..transport_proto::Response::default()
    }
}

fn error_code_response(
    request_id: Vec<u8>,
    error_code: transport_proto::response::ErrorCode,
) -> transport_proto::Response {
    transport_proto::Response {
        id: request_id,
        error_code: error_code.into(),
        ..transport_proto::Response::default()
    }
}

fn error_response(request_id: Vec<u8>, error: &failure::Fail) -> transport_proto::Response {
    transport_proto::Response {
        id: request_id,
        error_code: transport_proto::response::ErrorCode::Runtime.into(),
        error: Some(to_proto_error(error)),
        ..transport_proto::Response::default()
    }
}

fn to_proto_error(error: &failure::Fail) -> transport_proto::response::Error {
    transport_proto::response::Error {
        message: error.to_string(),
        cause: error.cause().map(to_proto_error).map(Box::new),
        backtrace: error
            .backtrace()
            .map(to_proto_backtrace)
            .unwrap_or_else(|| vec![]),
    }
}

fn to_proto_backtrace(backtrace: &failure::Backtrace) -> Vec<String> {
    backtrace
        .to_string()
        .lines()
        .map(|s| s.to_owned())
        .collect()
}

fn encode<M>(message: M) -> error::Result<bytes::Bytes>
where
    M: prost::Message,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.freeze())
}
