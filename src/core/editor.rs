use std::fs;

use futures;
use slog;

use error;
use schema::se::service as service_proto;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub struct SemanticEditor {
    log: slog::Logger,
}

impl SemanticEditor {
    pub fn new(log: slog::Logger) -> Self {
        SemanticEditor { log }
    }
}

impl service_proto::SemanticEditor for SemanticEditor {
    type Error = error::NestedError;
    type FetchSlateSchemaFuture = Box<
        ::futures::Future<Item = service_proto::FetchSlateSchemaResponse, Error = Self::Error>
            + Send,
    >;
    type ListFilesFuture =
        Box<::futures::Future<Item = service_proto::ListFilesResponse, Error = Self::Error> + Send>;

    fn fetch_slate_schema(
        &self,
        _input: service_proto::FetchSlateSchemaRequest,
    ) -> Self::FetchSlateSchemaFuture {
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
                        }).collect(),
                ).map(|file| service_proto::ListFilesResponse { file })
                .map_err(error::nested_error),
            ),
            Err(err) => Box::new(futures::future::err(error::nested_error(
                error::Error::from(err),
            ))),
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
