use std::io;
use std::result;

use failure;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", error)]
    Io {
        #[cause]
        error: io::Error,
        backtrace: failure::Backtrace,
    },
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io {
            error,
            backtrace: failure::Backtrace::new(),
        }
    }
}
