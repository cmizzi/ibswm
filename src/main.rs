mod window_manager;

use std::error::Error;
use std::process::exit;

use x11::connection::Connection;
use crate::window_manager::WindowManager;

fn main() -> Result<(), Box<dyn Error>> {
    // Create the main connection to X11 though the selected driver (libx11 or xcb).
    let (connection, screen_num) = x11::connect(None).expect("Unable to connect to X11 server.");

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