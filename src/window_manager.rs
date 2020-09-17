use std::error::Error;

use x11::x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection as X11Connection,
    errors::{
        ReplyError,
        ReplyOrIdError,
    },
    protocol::Event,
    protocol::xproto::ACCESS_ERROR,
    protocol::xproto::ChangeWindowAttributesAux,
    protocol::xproto::ConfigureRequestEvent,
    protocol::xproto::ConfigureWindowAux,
    protocol::xproto::ConfigWindow,
    protocol::xproto::ConnectionExt,
    protocol::xproto::CreateWindowAux,
    protocol::xproto::EventMask,
    protocol::xproto::GetGeometryReply,
    protocol::xproto::MapRequestEvent,
    protocol::xproto::Screen,
    protocol::xproto::SetMode,
    protocol::xproto::UnmapNotifyEvent,
    protocol::xproto::Window,
    protocol::xproto::WindowClass,
};
#[allow(unused_imports)]
use x11::x11rb::protocol::randr::{self, ConnectionExt as _};
use x11::connection::Connection;

pub struct WindowManager<'a, C: X11Connection> {
    connection: &'a mut Connection<'a, C>,
}

impl<'a, C: X11Connection> WindowManager<'a, C> {
    /// Create a new WindowManager instance.
    pub fn new(connection: &'a mut Connection<'a, C>) -> Result<WindowManager<'a, C>, ReplyOrIdError> {
        let wm = Self {
            connection
        };

        Ok(wm)
    }

    /// Try to change a root window property to assert another WM is not running.
    pub fn check_if_another_wm_is_running(&self) -> Result<(), Box<dyn Error>> {
        // Create a change and mask event to obtain lock.
        let change = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::SubstructureRedirect | EventMask::SubstructureNotify);

        // Try to take control over the root window.
        match self.connection.conn.change_window_attributes(self.connection.screen.root, &change)?.check() {
            // Another WM is running.
            Err(ReplyError::X11Error(error)) if error.error_code() == ACCESS_ERROR => {
                Err("It seems another WM is already running.".into())
            }

            // An error occurred while trying to update attributes.
            Err(error) => {
                Err(format!("Error: {:?}", error).into())
            }

            // All's good, we're good to go!
            Ok(_) => {
                Ok(())
            }
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>>{
        loop {
            self.connection.gracefully_destroy_window(
                self.connection.screen.root
            )?;

            break;
        }

        Ok(())
    }
}
