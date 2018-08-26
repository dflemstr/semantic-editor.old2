//! The core of the editor.
//!
//! Responsible for running the infrastructure used by all of the editor front-ends.
#![allow(unused_qualifications)]

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
    use structopt::StructOpt;

    let options = options::Options::from_args();

    let log = logger::init(&options);

    let log = version::init(log);

    info!(log, "Parsed command-line options";
    "options" => format!("{:?}", options));

    let editor = editor::SemanticEditor::new(log.new(o!("component" => "editor")));
    let server_handler = service_proto::SemanticEditorServer::new(editor);
    let server = server::Server::new(log.new(o!("component" => "server")), server_handler);
    server.run()?;

    info!(log, "program is terminating");
    Ok(())
}
