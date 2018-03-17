//! Common RPC definitions for various communication protocols.
//!
//! These RPC definitions can be used for a wide variety of transport protocols as long as they can
//! agree on using protobuf-derived message schemata.
use std::any;

use bytes;
use futures;
use prost;

pub mod error;
pub mod websocket;

/// A descriptor for an available RPC service.
pub trait ServiceDescriptor {
    /// The associated type of method descriptors.
    type Method: MethodDescriptor;

    /// The name of the service, used in Rust code and perhaps for human readability.
    fn name() -> &'static str;

    /// The raw protobuf name of the service.
    fn proto_name() -> &'static str;

    /// All of the available methods on the service.
    fn methods() -> &'static [Self::Method];
}

/// A descriptor for a method available on an RPC service.
pub trait MethodDescriptor: Copy {
    /// The name of the service, used in Rust code and perhaps for human readability.
    fn name(&self) -> &'static str;

    /// The raw protobuf name of the service.
    fn proto_name(&self) -> &'static str;

    /// The Rust `TypeId` for the input that this method accepts.
    fn input_type(&self) -> any::TypeId;

    /// The raw protobuf name for the input type that this method accepts.
    fn input_proto_type(&self) -> &'static str;

    /// The Rust `TypeId` for the output that this method produces.
    fn output_type(&self) -> any::TypeId;

    /// The raw protobuf name for the output type that this method produces.
    fn output_proto_type(&self) -> &'static str;
}

/// A server implementation for a specific service descriptor.
pub trait Server<A, S>
where
    A: Send,
    S: ServiceDescriptor + 'static,
{
    /// Handles a particular method from the service.
    ///
    /// `input` must contain bytes representing a valid encoding for this particular method's input
    /// protobuf type.
    fn handle(
        &self,
        method: S::Method,
        handler: A,
        input: bytes::Bytes,
    ) -> Box<futures::Future<Item = bytes::Bytes, Error = error::Error> + Send>;
}

/// A client implementation for a particular transport protocol.
///
/// This can be used to encode requests to a particular upstream service.
pub trait Client {
    /// Perform a raw call to the specified service and method.
    fn call(
        &mut self,
        service_name: &str,
        method_name: &str,
        input: bytes::Bytes,
    ) -> Box<futures::Future<Item = bytes::Bytes, Error = error::Error> + Send>;
}

/// Efficiently decode a particular message type from a byte buffer.
///
/// Mostly used from generated code.
pub fn decode<B, M>(buf: B) -> error::Result<M>
where
    B: bytes::IntoBuf,
    M: prost::Message + Default,
{
    let message = prost::Message::decode(buf)?;
    Ok(message)
}

/// Efficiently encode a particular message into a byte buffer.
///
/// Mostly used from generated code.
pub fn encode<M>(message: M) -> error::Result<bytes::Bytes>
where
    M: prost::Message,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.freeze())
}
