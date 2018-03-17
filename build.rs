extern crate prost_build;
extern crate vergen;

use std::fmt;

fn main() {
    prost_build::Config::new()
        .service_generator(Box::new(ServiceGenerator))
        .compile_protos(
            &[
                "src/schema/se/action/action.proto",
                "src/schema/se/data/format.proto",
                "src/schema/se/service/service.proto",
                "src/schema/se/websocket/websocket.proto",
            ],
            &["src/schema"],
        )
        .unwrap();

    vergen::vergen(vergen::OutputFns::all()).unwrap();
}

struct ServiceGenerator;

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&self, service: prost_build::Service, mut buf: &mut String) {
        use std::fmt::Write;

        let mut trait_methods = String::new();
        let mut enum_methods = String::new();
        let mut list_enum_methods = String::new();
        let mut match_name_methods = String::new();
        let mut match_proto_name_methods = String::new();
        let mut match_input_type_methods = String::new();
        let mut match_input_proto_type_methods = String::new();
        let mut match_output_type_methods = String::new();
        let mut match_output_proto_type_methods = String::new();
        let mut match_handle_methods = String::new();

        for method in service.methods {
            assert!(
                !method.client_streaming,
                "Client streaming not yet supported for method {}",
                method.proto_name
            );
            assert!(
                !method.server_streaming,
                "Server streaming not yet supported for method {}",
                method.proto_name
            );

            ServiceGenerator::write_comments(&mut trait_methods, 4, &method.comments).unwrap();
            writeln!(
                trait_methods,
                r#"    fn {name}(&self, input: {input_type})
                   -> Box<::futures::Future<Item = {output_type},
                                            Error = ::rpc::error::Error>>;"#,
                name = method.name,
                input_type = method.input_type,
                output_type = method.output_type
            ).unwrap();

            ServiceGenerator::write_comments(&mut enum_methods, 4, &method.comments).unwrap();
            writeln!(enum_methods, "    {name},", name = method.proto_name).unwrap();
            writeln!(
                list_enum_methods,
                "            {service_name}MethodDescriptor::{name},",
                service_name = service.name,
                name = method.proto_name
            ).unwrap();

            let case = format!(
                "            {service_name}MethodDescriptor::{name} => ",
                service_name = service.name,
                name = method.proto_name
            );

            writeln!(match_name_methods, "{}{:?},", case, method.name).unwrap();
            writeln!(match_proto_name_methods, "{}{:?},", case, method.proto_name).unwrap();
            writeln!(
                match_input_type_methods,
                "{}::std::any::TypeId::of::<{}>(),",
                case, method.input_type
            ).unwrap();
            writeln!(
                match_input_proto_type_methods,
                "{}{:?},",
                case, method.input_proto_type
            ).unwrap();
            writeln!(
                match_output_type_methods,
                "{}::std::any::TypeId::of::<{}>(),",
                case, method.output_type
            ).unwrap();
            writeln!(
                match_output_proto_type_methods,
                "{}{:?},",
                case, method.output_proto_type
            ).unwrap();
            write!(
                match_handle_methods,
                r#"{}
                Box::new(
                    ::futures::future::result(::rpc::decode(input))
                        .and_then(move |i| handler.{name}(i))
                        .and_then(::rpc::encode)),
"#,
                case,
                name = method.name
            ).unwrap();
        }

        ServiceGenerator::write_comments(&mut buf, 0, &service.comments).unwrap();
        write!(
            buf,
            r#"pub trait {name} {{
{trait_methods}}}
/// A service descriptor for a `{name}`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct {descriptor_name};
/// A server for a `{name}`.
///
/// This implements the `Server` trait by handling requests and dispatch them to methods on the
/// supplied `{name}`.
#[derive(Clone, Debug)]
pub struct {server_name}<A>(A) where A: {name};
/// A client for a `{name}`.
///
/// This implements the `{name}` trait by dispatching all method calls to the supplied `Client`.
pub struct {client_name}<C>(C) where C: ::rpc::Client;
/// A method available on a `{name}`.
///
/// This can be used as a key when routing requests for servers/clients of a `{name}`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum {method_descriptor_name} {{
{enum_methods}}}
impl<A> {server_name}<A> where A: {name} {{
    pub fn new(handler: A) -> {server_name}<A> {{
        {server_name}(handler)
    }}
}}
impl ::rpc::ServiceDescriptor for {descriptor_name} {{
    type Method = {method_descriptor_name};
    fn name() -> &'static str {{ {name:?} }}
    fn proto_name() -> &'static str {{ {proto_name:?} }}
    fn methods() -> &'static [Self::Method] {{
        &[
{list_enum_methods}        ]
    }}
}}
impl<A> ::rpc::Server<A, {descriptor_name}> for {server_name}<A> where A: {name} + 'static {{
    fn handle(
        &self,
        method: {method_descriptor_name},
        handler: A,
        input: ::bytes::Bytes)
        -> Box<::futures::Future<Item = ::bytes::Bytes, Error = ::rpc::error::Error> + 'static>
    {{
        use futures::Future;

        match method {{
{match_handle_methods}        }}
    }}
}}
impl ::rpc::MethodDescriptor for {method_descriptor_name} {{
    fn name(&self) -> &'static str {{
        match *self {{
{match_name_methods}        }}
    }}
    fn proto_name(&self) -> &'static str {{
        match *self {{
{match_proto_name_methods}        }}
    }}
    fn input_type(&self) -> ::std::any::TypeId {{
        match *self {{
{match_input_type_methods}        }}
    }}
    fn input_proto_type(&self) -> &'static str {{
        match *self {{
{match_input_proto_type_methods}        }}
    }}
    fn output_type(&self) -> ::std::any::TypeId {{
        match *self {{
{match_output_type_methods}        }}
    }}
    fn output_proto_type(&self) -> &'static str {{
        match *self {{
{match_output_proto_type_methods}        }}
    }}
}}
"#,
            name = service.name,
            descriptor_name = format!("{}Descriptor", service.name),
            server_name = format!("{}Server", service.name),
            client_name = format!("{}Client", service.name),
            method_descriptor_name = format!("{}MethodDescriptor", service.name),
            proto_name = service.proto_name,
            trait_methods = trait_methods,
            enum_methods = enum_methods,
            list_enum_methods = list_enum_methods,
            match_name_methods = match_name_methods,
            match_proto_name_methods = match_proto_name_methods,
            match_input_type_methods = match_input_type_methods,
            match_input_proto_type_methods = match_input_proto_type_methods,
            match_output_type_methods = match_output_type_methods,
            match_output_proto_type_methods = match_output_proto_type_methods,
            match_handle_methods = match_handle_methods
        ).unwrap();
    }
}

impl ServiceGenerator {
    fn write_comments<W>(
        mut write: W,
        indent: usize,
        comments: &prost_build::Comments,
    ) -> fmt::Result
    where
        W: fmt::Write,
    {
        for comment in &comments.leading {
            for line in comment.lines().filter(|s| !s.is_empty()) {
                writeln!(write, "{}///{}", " ".repeat(indent), line)?;
            }
        }
        Ok(())
    }
}
