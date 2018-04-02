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
    if env::var("TERM")
        .map(|s| s.starts_with("xterm"))
        .unwrap_or(false)
    {
        env::set_var("TERM", "xterm");
    }

    if options.color {
        builder = builder.force_color();
    }

    let decorator = builder.build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();

    let log = config_log_0(drain, &options);

    slog_scope::set_global_logger(log.clone()).cancel_reset();
    slog_stdlog::init().unwrap();
    log
}

fn config_log_0<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    if options.silent {
        config_log_1(slog::Discard, options)
    } else {
        config_log_1(drain, options)
    }
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
        config_log_3(
            slog::Duplicate::new(drain, journald_drain).ignore_res(),
            options,
        )
    } else {
        config_log_3(drain, options)
    }
}

#[cfg(not(feature = "journald"))]
fn config_log_2<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    config_log_3(drain, options)
}

fn config_log_3<D>(drain: D, options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    use slog::Drain;
    if options.debug {
        config_log_final(drain, options)
    } else {
        let total = options.verbose as i32 - options.quiet as i32;

        let level = match total {
            n if n < -3 => return config_log_final(slog::Discard, options),
            -3 => slog::Level::Critical,
            -2 => slog::Level::Error,
            -1 => slog::Level::Warning,
            0 => slog::Level::Info,
            1 => slog::Level::Debug,
            _ => slog::Level::Trace,
        };

        config_log_final(slog::LevelFilter::new(drain, level).ignore_res(), options)
    }
}

fn config_log_final<D>(drain: D, _options: &super::options::Options) -> slog::Logger
where
    D: slog::Drain<Ok = (), Err = slog::Never> + Send + 'static,
{
    use slog::Drain;
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}
