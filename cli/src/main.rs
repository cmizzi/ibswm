use clap::Clap;
use std::error::Error;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
struct Opts {
    /// Verbosity. By default, will only log ERROR level.
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    #[clap(subcommand)]
    command: SubCommand
}

#[derive(Clap, Debug)]
enum SubCommand {
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
struct Config {
    /// Apply configuration on a specific monitor.
    #[clap(short, long)]
    monitor: Option<String>,

    /// Apply configuration on a specific monitor.
    key: Option<String>,

    /// Apply configuration on a specific monitor.
    value: Option<String>,
}

#[derive(Clap, Debug)]
struct Monitor {
    /// Monitor name.
    #[clap(short, long)]
    monitor: Option<String>,

    /// Custom monitor name.
    #[clap(short, long)]
    name: Option<String>,

    /// List of desktops to create.
    desktops: Vec<String>,
}

#[derive(Clap, Debug)]
struct Rule {
    /// Default states to define on new application matching the <application> name.
    #[clap(short, long)]
    state: Option<Vec<String>>,

    /// Default desktop to map new application matching the <application> name.
    #[clap(short, long)]
    desktop: Option<String>,

    /// Application X11 name.
    application: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    match opts.command {
        SubCommand::Config(_) => println!("Configure."),
        SubCommand::Monitor(_) => println!("Monitor."),
        SubCommand::Rule(_) => println!("Rule."),
    }

    Ok(())
}