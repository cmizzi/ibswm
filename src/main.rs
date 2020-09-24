#[macro_use]
extern crate log;

use std::error::Error;
use std::io::prelude::*;
use std::io::{Write, BufReader};
use std::os::unix::net::UnixListener;
use std::os::unix::io::AsRawFd;
use std::process::exit;

use clap::Clap;
use env_logger::Env;
use nix::sys::select::{FdSet, select};
use x11rb::xcb_ffi::XCBConnection;

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

    #[clap(short, long, default_value = "/tmp/ibswm.sock")]
    socket: String,
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

    // Create the main connection to X11 though the selected driver through XCB.
    let (connection, screen_num) = XCBConnection::connect(None)
        .expect("Unable to connect to X11 server.");

    let mut connection = Connection::new(&connection, screen_num);
    let mut wm = WindowManager::new(&mut connection)?;

    // Assert that another WM is not running, because we won't be able to take control over X11
    // server.
    if let Err(error) = wm.check_if_another_wm_is_running() {
        println!("{}", error);
        exit(1);
    }

    // Define am empty file descriptor set. This set is defined to detect file description read
    // changes.
    let mut descriptors = FdSet::new();

    // Connect to the CLI unix socket in order to receive order from `ibsc`.
    // First, we need to remove any existing socket to prevent `Already In Use` error.
    std::fs::remove_file(&opts.socket).ok();

    let stream = UnixListener::bind(&opts.socket)?;
    stream.set_nonblocking(true).expect("Couldn't set non-blocking socket mode.");

    loop {
        wm.flush()?;

        // On each loop, clear set of file descriptors and re-insert them.
        descriptors.clear();
        descriptors.insert(stream.as_raw_fd());
        descriptors.insert(wm.connection_fd());

        // We need to check if any of file descriptor has changed because we don't want to loop
        // forever. The goal here is to prevent as many as possible empty loops (CPU usage).
        //
        // As this statement is blocking, we can safely wait. Also, this method will mutate
        // descriptors to only contains ready file descriptors.
        if select(descriptors.highest().unwrap() + 1, &mut descriptors, None, None, None)? > 0 {
            // Applied only when the CLI communicates.
            if descriptors.contains(stream.as_raw_fd()) {
                debug!("Reading command socket.");

                // Iterate over clients. This method will not block because we called
                // `set_nonblocking` method on the socket. But, this method will block until the
                // client disconnects.
                if let Ok((stream, _)) = stream.accept() {
                    let stream = BufReader::new(stream);

                    for line in stream.lines() {
                        wm.handle_command(line.unwrap());
                    }
                }
            }

            // Applied only when X11 server fires events.
            if descriptors.contains(wm.connection_fd()) {
                debug!("Polling X11 events.");

                while let Some(event) = wm.poll_for_event()? {
                    wm.handle_event(event)?;
                }
            }
        }
    }
}