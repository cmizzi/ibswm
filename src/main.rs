#[macro_use]
extern crate log;

use std::error::Error;
use std::io::Write;
use std::process::exit;

use clap::Clap;
use env_logger::Env;

use crate::connection::Connection;
use crate::window_manager::WindowManager;

mod window_manager;
mod desktop;
mod client;
mod connection;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Cyril Mizzi <me@p1ngouin.com>")]
struct Opts {
    /// Verbosity. By default, will only log ERROR level.
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

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

    // Create the main connection to X11 though the selected driver (libx11 or xcb).
    let (connection, screen_num) = x11rb::connect(None).expect("Unable to connect to X11 server.");

    let mut connection = Connection::new(&connection, screen_num);
    let mut wm = WindowManager::new(&mut connection)?;

    // Assert that another WM is not running, because we won't be able to take control over X11
    // server.
    if let Err(error) = wm.check_if_another_wm_is_running() {
        println!("{}", error);
        exit(1);
    }

    // Start the event loop.
    wm.run()
}