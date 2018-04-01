extern crate prost_build;
extern crate prost_simple_rpc_build;
extern crate vergen;

fn main() {
    prost_build::Config::new()
        .service_generator(Box::new(prost_simple_rpc_build::ServiceGenerator::new()))
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
