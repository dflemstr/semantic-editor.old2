#[derive(StructOpt, Debug)]
#[structopt(name = "se")]
pub struct Options {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: u8,

    /// Log to syslog (no-op unless this program was compiled with the "syslog" feature)
    #[structopt(long = "syslog")]
    pub syslog: bool,

    /// Log to journald (no-op unless this program was compiled with the "journald" feature)
    #[structopt(long = "journald")]
    pub journald: bool,
}
