//! Common error definitions.
use std::result;

use failure;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A> = result::Result<A, Error>;

/// An error has occurred, either remotely or locally.
pub type Error = failure::Error;
