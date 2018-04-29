use bytes;
use failure;
use futures;
use hyper;
use mime;
use prost;
use prost_simple_rpc;
use unicase;

use futures::prelude::{async, await};

use error;
use schema::se::transport as transport_proto;

mod standalone;

type RpcError = prost_simple_rpc::error::Error<error::NestedError>;

lazy_static! {
    static ref REQUEST_CONTENT_TYPE: hyper::header::ContentType = {
        use std::str::FromStr;
        hyper::header::ContentType(
            mime::Mime::from_str("application/x-semantic-editor-request").unwrap(),
        )
    };
    static ref RESPONSE_CONTENT_TYPE: hyper::header::ContentType = {
        use std::str::FromStr;
        hyper::header::ContentType(
            mime::Mime::from_str("application/x-semantic-editor-response").unwrap(),
        )
    };
}

pub struct Server<H>(H);

enum Body {
    FileStream(standalone::FileStream),
    Bytes(Option<bytes::Bytes>),
}

struct HandlerHyperService<H> {
    handler: H,
}

impl<H> Server<H>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    pub fn new(handler: H) -> Self {
        Server(handler)
    }

    pub fn run(self) -> error::Result<()> {
        let addr = "127.0.0.1:12345".parse()?;

        let server = hyper::server::Http::new().bind(&addr, move || {
            let handler = self.0.clone();
            Ok(HandlerHyperService { handler })
        })?;
        server.run()?;
        Ok(())
    }
}

impl<H> hyper::server::Service for HandlerHyperService<H>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    type Request = hyper::server::Request;
    type Response = hyper::server::Response<Body>;
    type Error = hyper::Error;
    type Future = Box<futures::Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: hyper::server::Request) -> Self::Future {
        Box::new(call_raw(self.handler.clone(), req))
    }
}

#[async]
fn call_raw<H>(
    handler: H,
    req: hyper::server::Request,
) -> Result<hyper::server::Response<Body>, hyper::Error>
where
    H: prost_simple_rpc::handler::Handler<Error = RpcError>,
{
    if req.method() == &hyper::Method::Options {
        let response = hyper::server::Response::new().with_status(hyper::StatusCode::Ok);
        Ok(with_cors_headers(response))
    } else if is_static_file_request(&req) {
        Ok(with_cors_headers(serve_static_file(&req)))
    } else if let Some(request) = await!(parse_rpc_request(req))? {
        await!(handle_request(handler, request))
    } else {
        Ok(hyper::server::Response::new().with_status(hyper::StatusCode::NotFound))
    }
}

#[async]
fn handle_request<H>(
    handler: H,
    request: transport_proto::Request,
) -> Result<hyper::server::Response<Body>, hyper::Error>
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
fn parse_rpc_request(
    req: hyper::server::Request,
) -> Result<Option<transport_proto::Request>, hyper::Error> {
    use futures::Stream;
    use prost::Message;

    if req.headers().get::<hyper::header::ContentType>() != Some(&REQUEST_CONTENT_TYPE) {
        return Ok(None);
    }

    let (path_service_name, path_method_name) = {
        let parts = req.path().split("/").collect::<Vec<_>>();
        if parts.len() == 3 && parts[0] == "" {
            (parts[1].to_owned(), parts[2].to_owned())
        } else {
            return Ok(None);
        }
    };

    let body = bytes::Bytes::from(await!(req.body().concat2())?);
    if let Some(request) = transport_proto::Request::decode(body).ok() {
        if request.service_name == path_service_name && request.method_name == path_method_name {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn is_static_file_request(request: &hyper::server::Request) -> bool {
    standalone::file_exists(canonicalize_path(request.path()))
}

fn serve_static_file(request: &hyper::server::Request) -> hyper::server::Response<Body> {
    let path = canonicalize_path(request.path());

    if request
        .headers()
        .get::<hyper::header::AcceptEncoding>()
        .map(|e| e.iter().any(|i| i.item == hyper::header::Encoding::Brotli))
        .unwrap_or(false)
    {
        let data = standalone::brotli_compressed_file(path).unwrap();
        hyper::server::Response::new()
            .with_status(hyper::StatusCode::Ok)
            .with_header(hyper::header::ContentEncoding(vec![
                hyper::header::Encoding::Brotli,
            ]))
            .with_body(Body::FileStream(data.contents))
    } else {
        let data = standalone::file(path).unwrap();
        hyper::server::Response::new()
            .with_status(hyper::StatusCode::Ok)
            .with_header(hyper::header::ContentLength(data.size))
            .with_body(Body::FileStream(data.contents))
    }
}

fn canonicalize_path(path: &str) -> &str {
    if path.is_empty() || path == "/" {
        "index.html"
    } else {
        &path[1..]
    }
}

fn hyper_response(response: transport_proto::Response) -> hyper::server::Response<Body> {
    let data = encode(response).unwrap();
    with_cors_headers(
        hyper::server::Response::new()
            .with_status(hyper::StatusCode::Ok)
            .with_header(RESPONSE_CONTENT_TYPE.clone())
            .with_header(hyper::header::ContentLength(data.len() as u64))
            .with_body(Body::Bytes(Some(data))),
    )
}

fn with_cors_headers<B>(response: hyper::server::Response<B>) -> hyper::server::Response<B> {
    response
        .with_header(hyper::header::AccessControlAllowOrigin::Any)
        .with_header(hyper::header::AccessControlAllowMethods(vec![
            hyper::Method::Post,
        ]))
        .with_header(hyper::header::AccessControlAllowHeaders(vec![
            unicase::Ascii::new("*".to_owned()),
        ]))
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

impl futures::Stream for Body {
    type Item = bytes::Bytes;
    type Error = hyper::Error;

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        match *self {
            Body::FileStream(ref mut stream) => stream.poll(),
            Body::Bytes(ref mut bytes) => Ok(futures::Async::Ready(bytes.take())),
        }
    }
}
