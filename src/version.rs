//! Build version information.
#![allow(dead_code)]

use slog;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub fn init(log: slog::Logger) -> slog::Logger {
    let log = log.new(o!(
        "version" => format!(concat!(env!("CARGO_PKG_VERSION"), "-{}"), VERGEN_SHA_SHORT),
        "target" => VERGEN_TARGET_TRIPLE,
    ));

    info!(
        log,
        concat!(
            "Initializing ",
            env!("CARGO_PKG_NAME"),
            " version ",
            env!("CARGO_PKG_VERSION"), "-{}"
        ), VERGEN_SHA_SHORT;
        "name" => env!("CARGO_PKG_NAME"),
        "created" => VERGEN_COMMIT_DATE,
        "built" => VERGEN_BUILD_TIMESTAMP,
    );

    log
}
