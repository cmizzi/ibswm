#[macro_use]
extern crate log;

use std::env;
use std::error::Error;
use std::io::Write;
use std::os::unix::net::UnixStream;

use clap::Clap;
use env_logger::Env;

use crate::lib::Opts;

mod lib;

fn current_exe() -> String {
    std::env::current_exe()
        .ok().unwrap()
        .file_name().unwrap()
        .to_str().unwrap()
        .to_owned()
}

/// Initialize the logger.
fn init_logger(opts: &Opts) {
    let app = current_exe();
    let env = Env::default().default_filter_or(
        match opts.verbose {
            0 => format!("{}=error", app),
            1 => format!("{}=info", app),
            2 => format!("{}=debug", app),
            _ => "trace".to_string(),
        }
    );

    env_logger::from_env(env)
        .format(|buf, record| {
            let level_style = buf.default_level_style(record.level());
            writeln!(buf, "[{} {:>5}]: {}", buf.timestamp(), level_style.value(record.level()), record.args())
        })
        .init();
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();
    init_logger(&opts);

    let mut stream = UnixStream::connect(&opts.socket).unwrap_or_else(|e| {
        error!("Cannot connect to the socket \"{}\" : {}.", opts.socket, e);
        std::process::exit(1);
    });

    let args: Vec<String> = env::args().collect();
    stream.write_all(args.join(" ").as_bytes())?;

    Ok(())
}