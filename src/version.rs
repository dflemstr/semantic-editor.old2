//! Build version information.
#![allow(dead_code)]

use slog;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub fn init(log: slog::Logger) -> slog::Logger {
    let log = log.new(o!(
        "version" => format!(concat!(env!("CARGO_PKG_VERSION"), "-{}"), short_sha()),
        "target" => target(),
    ));

    info!(
        log,
        concat!(
            "Initializing ",
            env!("CARGO_PKG_NAME"),
            " version ",
            env!("CARGO_PKG_VERSION")
        );
        "name" => env!("CARGO_PKG_NAME"),
        "created" => commit_date(),
        "built" => now(),
    );

    log
}
