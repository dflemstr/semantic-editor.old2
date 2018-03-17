use bytes;
use futures;
use futures::unsync::oneshot;
use prost;
use uuid;
use wasm_bindgen::prelude::*;
use std::collections;
use std::cell;
use std::rc;
use se::websocket as websocket_proto;

pub struct WebSocketRpc {
    websocket: WebSocket,
    handler: WebSocketHandler,
}

pub struct WebSocketFuture(oneshot::Receiver<bytes::Bytes>);

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct WebSocketHandler(rc::Rc<cell::RefCell<InnerWebSocketHandler>>);

#[derive(Default)]
struct InnerWebSocketHandler {
    outstanding: collections::HashMap<uuid::Uuid, oneshot::Sender<bytes::Bytes>>,
}

impl WebSocketRpc {
    pub fn open(url: &str) -> WebSocketRpc {
        use wasm_bindgen::convert::WasmBoundary;

        let websocket = WebSocket::new(url);
        let handler = WebSocketHandler::default();
        setWebSocketHandler(&websocket, handler.clone().into_js());
        WebSocketRpc { websocket, handler }
    }
}

impl super::Client for WebSocketRpc {
    fn call(
        &mut self,
        service_name: &str,
        method_name: &str,
        input: bytes::Bytes,
    ) -> Box<futures::Future<Item = bytes::Bytes, Error = super::error::Error> + 'static> {
        let id = uuid::Uuid::parse_str(&genUuid()).unwrap();

        let request = websocket_proto::Request {
            id: id.as_bytes().to_vec(),
            data: input.to_vec(),
            service_name: service_name.to_owned(),
            method_name: method_name.to_owned(),
        };

        let mut buffer = Vec::with_capacity(prost::Message::encoded_len(&request));
        // Should never fail since we are writing in-memory.
        prost::Message::encode(&request, &mut buffer).unwrap();

        let (tx, rx) = oneshot::channel();
        self.handler.0.borrow_mut().outstanding.insert(id, tx);
        self.websocket.send(&buffer);
        info!("Sent data: {:?}", buffer);
        Box::new(WebSocketFuture(rx))
    }
}

impl futures::Future for WebSocketFuture {
    type Item = bytes::Bytes;
    type Error = super::error::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self.0.poll() {
            Ok(futures::Async::Ready(data)) => {
                let response: websocket_proto::Response = prost::Message::decode(data)?;
                Ok(futures::Async::Ready(response.data.into()))
            }
            Ok(futures::Async::NotReady) => Ok(futures::Async::NotReady),
            Err(ref err) => Err((*err).into()),
        }
    }
}

#[wasm_bindgen]
impl WebSocketHandler {
    pub fn onclose(&self, web_socket: WebSocket, code: u16, reason: &str, was_clean: bool) {
        info!("onclose({:?}, {:?}, {:?})", code, reason, was_clean);
    }

    pub fn onerror(&self, web_socket: WebSocket) {
        info!("onerror()");
    }

    pub fn onmessage(&self, web_socket: WebSocket, data: &[u8], origin: &str) {
        info!("onmessage({:?}, {:?})", data, origin);
    }

    pub fn onopen(&self, web_socket: WebSocket) {
        info!("onopen()");
    }
}

#[wasm_bindgen(module = "rpc/websocket")]
extern "C" {
    #[allow(non_snake_case)]
    fn setWebSocketHandler(websocket: &WebSocket, handler: u32);

    #[allow(non_snake_case)]
    fn genUuid() -> String;
}

#[wasm_bindgen]
extern "C" {
    pub type WebSocket;
    type Event;

    #[wasm_bindgen(constructor)]
    fn new(url: &str) -> WebSocket;

    #[wasm_bindgen(method)]
    fn close(this: &WebSocket, code: u32, reason: &str);

    #[wasm_bindgen(method)]
    fn send(this: &WebSocket, data: &[u8]);

    #[wasm_bindgen(method, getter)]
    #[allow(non_snake_case)]
    fn binaryType(this: &WebSocket) -> String;

    #[wasm_bindgen(method, setter)]
    #[allow(non_snake_case)]
    fn set_binaryType(this: &WebSocket, value: &str) -> String;

    #[wasm_bindgen(method, getter)]
    #[allow(non_snake_case)]
    fn bufferedAmount(this: &WebSocket) -> u32;

    #[wasm_bindgen(method, getter)]
    fn extensions(this: &WebSocket) -> String;
}
