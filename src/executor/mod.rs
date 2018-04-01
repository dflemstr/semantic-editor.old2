//! Execution facilities for running asynchronous code.
use futures;

pub mod sys;

/// Runs a future to completion by instantiating an executor that's appropriate for the current
/// target platform.  Behaves mostly like [`tokio::run`], but might not block until the future has
/// finished.
pub fn run<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    sys::run(future)
}

/// Spawns a future on the default executor.  This only works while a default executor is active;
/// for example from within another future.  Behaves mostly like [`tokio::spawn`].
pub fn spawn<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    sys::spawn(future)
}
