//! The "default" executor implementation that delegates to the `tokio` crate.
use futures;
use tokio;

/// Start the Tokio runtime using the supplied future to bootstrap execution.
///
/// This function is used to bootstrap the execution of a Tokio application. It
/// does the following:
///
/// * Start the Tokio runtime using a default configuration.
/// * Spawn the given future onto the thread pool.
/// * Block the current thread until the runtime shuts down.
///
/// Note that the function will not return immediately once `future` has
/// completed. Instead it waits for the entire runtime to become idle.
///
/// # Panics
///
/// This function panics if called from the context of an executor.
pub fn run<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    tokio::run(future);
}

/// Spawns a future on the default executor.
///
/// In order for a future to do work, it must be spawned on an executor. The
/// `spawn` function is the easiest way to do this. It spawns a future on the
/// default executor for the current execution context (tracked using a
/// thread-local variable).
///
/// The default executor is **usually** a thread pool.
///
/// # Panics
///
/// This function will panic if the default executor is not set or if spawning
/// onto the default executor returns an error. To avoid the panic, use
/// `DefaultExecutor`.
pub fn spawn<F>(future: F)
where
    F: futures::Future<Item = (), Error = ()> + Send + 'static,
{
    tokio::spawn(future);
}
