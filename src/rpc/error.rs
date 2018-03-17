//! Error type definitions for errors that can occur during RPC interactions.
use std::result;

use failure;
use futures;
use prost;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A> = result::Result<A, Error>;

/// An error has occurred.
#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred during input decoding.
    #[fail(display = "Decode error: {}", error)]
    Decode {
        /// The underlying decode error.
        #[cause]
        error: prost::DecodeError,
        /// The backtrace to where the error occurred.
        backtrace: failure::Backtrace,
    },
    /// An error occurred during output encoding.
    #[fail(display = "Encode error: {}", error)]
    Encode {
        /// The underlying encode error.
        #[cause]
        error: prost::EncodeError,

        /// The backtrace to where the error occurred.
        backtrace: failure::Backtrace,
    },
    /// An async cancellation occurred.
    #[fail(display = "Canceled error: {}", error)]
    Canceled {
        /// The underlying canceled error.
        #[cause]
        error: futures::Canceled,

        /// The backtrace to where the error occurred.
        backtrace: failure::Backtrace,
    },
}

impl From<prost::DecodeError> for Error {
    fn from(error: prost::DecodeError) -> Self {
        Error::Decode {
            error,
            backtrace: failure::Backtrace::new(),
        }
    }
}

impl From<prost::EncodeError> for Error {
    fn from(error: prost::EncodeError) -> Self {
        Error::Encode {
            error,
            backtrace: failure::Backtrace::new(),
        }
    }
}

impl From<futures::Canceled> for Error {
    fn from(error: futures::Canceled) -> Self {
        Error::Canceled {
            error,
            backtrace: failure::Backtrace::new(),
        }
    }
}
