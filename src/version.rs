//! Build version information.
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/version.rs"));

pub fn log() {
    info!(
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
        target()
    );
}
