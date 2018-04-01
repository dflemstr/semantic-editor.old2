#[derive(StructOpt, Debug)]
#[structopt(name = "se")]
pub struct Options {
    /// Activate debug mode, which logs everything.
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// Activate silent mode, which logs nothing (not even errors).  However, other log facilities
    /// (such as 'syslog' or 'journald') still work.
    #[structopt(short = "s", long = "silent")]
    pub silent: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.).  The verbose level minus the quiet level determines the
    /// final log verbosity.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// Quiet mode (-q, -qq, -qqq, etc.).  The verbose level minus the quiet level determines the
    /// final log verbosity.
    #[structopt(short = "q", long = "quiet", parse(from_occurrences))]
    pub quiet: u8,

    /// Force colorful output.
    #[structopt(long = "color")]
    pub color: bool,

    /// Log to syslog (no-op unless this program was compiled with the "syslog" feature)
    #[structopt(long = "syslog")]
    pub syslog: bool,

    /// Log to journald (no-op unless this program was compiled with the "journald" feature)
    #[structopt(long = "journald")]
    pub journald: bool,
}
