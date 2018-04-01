//! Build version information.
#![allow(dead_code)]

use slog;

include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub fn init(log: &slog::Logger) {
    let log = log.new(o!(
        "name" => env!("CARGO_PKG_NAME"),
        "version" => format!(concat!(env!("CARGO_PKG_VERSION"), "-{}"), short_sha()),
        "created" => commit_date(),
        "built" => now(),
        "target" => target()
    ));

    info!(
        log,
        concat!(
            "Initializing ",
            env!("CARGO_PKG_NAME"),
            " version ",
            env!("CARGO_PKG_VERSION"),
            "-{} created {} built {} running on {}"
        ),
        short_sha(),
        commit_date(),
        now(),
        target(),
    );
}
