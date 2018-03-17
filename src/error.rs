//! Common error definitions.
use std::io;
use std::result;

use failure;

use rpc;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A> = result::Result<A, Error>;

/// An error has occurred.
#[derive(Debug, Fail)]
pub enum Error {
    /// An IO error has occurred.
    #[fail(display = "IO error: {}", error)]
    Io {
        /// The underlying IO error.
        #[cause]
        error: io::Error,
        /// The backtrace to where the error occurred.
        backtrace: failure::Backtrace,
    },
    /// An RPC error has occurred.
    #[fail(display = "RPC error: {}", error)]
    Rpc {
        /// The underlying RPC error.
        #[cause]
        error: rpc::error::Error,
        /// The backtrace to where the error occurred.
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

impl From<rpc::error::Error> for Error {
    fn from(error: rpc::error::Error) -> Self {
        Error::Rpc {
            error,
            backtrace: failure::Backtrace::new(),
        }
    }
}
