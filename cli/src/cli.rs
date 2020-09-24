/// This part of the program is shared between `ibsc` and `ibswm` to easily manage command. If you
/// modify this part, please also check the `ibswm` package.

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
pub struct Opts {
    /// Verbosity. By default, will only log ERROR level.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,

    #[clap(short, long, default_value = "/tmp/ibswm.sock")]
    pub socket: String,

    #[clap(subcommand)]
    pub command: SubCommand
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Configure a monitor.
    #[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
    Config(Config),

    /// Define a monitor.
    #[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
    Monitor(Monitor),

    /// Apply a window rule.
    #[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
    Rule(Rule),
}

#[derive(Clap, Debug)]
pub struct Config {
    /// Apply configuration on a specific monitor.
    #[clap(short, long)]
    pub monitor: Option<String>,

    /// Apply configuration on a specific monitor.
    pub key: String,

    /// Apply configuration on a specific monitor.
    pub value: Option<String>,
}

#[derive(Clap, Debug)]
pub struct Monitor {
    /// Monitor name.
    pub monitor: String,

    /// Custom monitor name.
    #[clap(short, long)]
    pub name: Option<String>,

    /// List of desktops to create.
    #[clap(short, long)]
    pub desktops: Vec<String>,
}

#[derive(Clap, Debug)]
pub struct Rule {
    /// Default states to define on new application matching the <application> name.
    #[clap(short, long)]
    pub state: Option<Vec<String>>,

    /// Default desktop to map new application matching the <application> name.
    #[clap(short, long)]
    pub desktop: Option<String>,

    /// Application X11 name.
    pub application: String,
}
