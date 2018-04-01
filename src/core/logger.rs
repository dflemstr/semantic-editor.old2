use std::env;

use slog;
use slog_async;
#[cfg(feature = "journald")]
use slog_journald;
use slog_term;
use slog_scope;
use slog_stdlog;
#[cfg(feature = "syslog")]
use slog_syslog;

pub fn init(options: &super::options::Options) -> slog::Logger {
    use slog::Drain;

    let mut builder = slog_term::TermDecorator::new();

    // Work-around 'term' issue; for example lacking 256color support
    if env::var("TERM").map(|s| s.starts_with("xterm")).unwrap_or(false) {
        env::set_var("TERM", "xterm");
    }

    if options.color {
        builder = builder.force_color();
    }

    let decorator = builder.build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();

    let log = config_log_1(drain, &options);
    slog_scope::set_global_logger(log.clone()).cancel_reset();
    slog_stdlog::init().unwrap();
    log
}

#[cfg(feature = "syslog")]
fn config_log_1<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    if options.syslog {
        use slog::Drain;

        let syslog_drain = slog_syslog::unix_3164(slog_syslog::Facility::LOG_USER).unwrap();
        config_log_2(
            slog::Duplicate::new(drain, syslog_drain).ignore_res(),
            options,
        )
    } else {
        config_log_2(drain, options)
    }
}

#[cfg(not(feature = "syslog"))]
fn config_log_1<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    config_log_2(drain, options)
}

#[cfg(feature = "journald")]
fn config_log_2<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    if options.journald {
        use slog::Drain;
        let journald_drain = slog_journald::JournaldDrain;
        config_log_final(
            slog::Duplicate::new(drain, journald_drain).ignore_res(),
            options,
        )
    } else {
        config_log_final(drain, options)
    }
}

#[cfg(not(feature = "journald"))]
fn config_log_2<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    config_log_final(drain, options)
}

fn config_log_final<D>(drain: D, _options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    use slog::Drain;
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}
