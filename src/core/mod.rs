//! The core of the editor.
//!
//! Responsible for running the infrastructure used by all of the editor front-ends.
#![allow(unused_qualifications)]

use std::collections;

use error;
use schema::se::service as service_proto;
use version;

mod editor;
mod logger;
mod options;
mod server;

/// Runs the semantic editor core.
///
/// This can be treated like a main function; it will parse command line arguments etc.
pub fn run() -> error::Result<()> {
    use semantic::Semantic;
    use structopt::StructOpt;

    let mut classes = collections::HashMap::new();
    ::data::Data::visit_classes(&mut |class| classes.insert(class.id.clone(), class).is_none());
    for class in classes.into_iter() {
        println!("{:?}", class);
    }

    let options = options::Options::from_args();

    let log = logger::init(&options);

    let log = version::init(log);

    info!(log, "Parsed command-line options";
    "options" => format!("{:?}", options));

    let editor = editor::SemanticEditor::new(log.new(o!("component" => "editor")));
    let server_handler = service_proto::SemanticEditorServer::new(editor);
    let server = server::Server::new(server_handler);
    server.run()?;

    info!(log, "program is terminating");
    Ok(())
}
