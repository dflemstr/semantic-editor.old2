use std::fs;

use failure;
use futures;
use slog;

use error;
use schema::se::service as service_proto;

#[derive(Clone, Debug)]
pub struct SemanticEditor {
    log: slog::Logger,
}

impl SemanticEditor {
    pub fn new(log: slog::Logger) -> Self {
        SemanticEditor { log }
    }
}

impl service_proto::SemanticEditor for SemanticEditor {
    type Error = failure::Compat<failure::Error>;
    type PerformActionFuture =
        Box<::futures::Future<Item = service_proto::ActionResponse, Error = Self::Error> + Send>;
    type ListFilesFuture =
        Box<::futures::Future<Item = service_proto::ListFilesResponse, Error = Self::Error> + Send>;

    fn perform_action(&self, input: service_proto::ActionRequest) -> Self::PerformActionFuture {
        info!(self.log, "perform_action called");
        unimplemented!()
    }

    fn list_files(&self, input: service_proto::ListFilesRequest) -> Self::ListFilesFuture {
        use futures::Future;
        use schema::se::service::list_files_response;

        info!(self.log, "list_files called");
        match fs::read_dir(input.path) {
            Ok(read_dir) => Box::new(
                futures::future::result(
                    read_dir
                        .map(|entry| {
                            let entry = entry?;
                            Ok(list_files_response::File {
                                path: entry.path().to_string_lossy().into_owned(),
                                kind: to_kind(entry.file_type()?),
                            })
                        })
                        .collect(),
                ).map(|file| service_proto::ListFilesResponse { file })
                    .map_err(|e: error::Error| e.compat()),
            ),
            Err(err) => Box::new(futures::future::err(error::Error::from(err).compat())),
        }
    }
}

fn to_kind(ty: fs::FileType) -> Option<service_proto::list_files_response::file::Kind> {
    use schema::se::service::list_files_response::file::Kind;

    if ty.is_dir() {
        Some(Kind::Directory(Default::default()))
    } else if ty.is_file() {
        Some(Kind::Regular(Default::default()))
    } else if ty.is_symlink() {
        Some(Kind::Link(Default::default()))
    } else {
        // TODO: support UNIX-y kinds
        None
    }
}