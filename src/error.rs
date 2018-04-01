//! Common error definitions.
use std::fmt;
use std::result;

use failure;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A> = result::Result<A, Error>;

/// An error has occurred, either remotely or locally.
pub type Error = failure::Error;

/// A nested, untyped error.
///
/// This type is useful when interacting with other crates that require errors to implement `Fail`.
#[derive(Debug)]
pub struct NestedError(Error);

/// Create a new nested error from the specified `Error`.
pub fn nested_error(error: Error) -> NestedError {
    NestedError(error)
}

impl failure::Fail for NestedError {
    fn cause(&self) -> Option<&failure::Fail> {
        Some(self.0.cause())
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        Some(self.0.backtrace())
    }
}

impl fmt::Display for NestedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error> for NestedError {
    fn from(error: Error) -> Self {
        NestedError(error)
    }
}
