use std::error::Error;
use std::fmt::{Debug, Formatter};

use x11rb::{
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
    protocol::xproto::ConnectionExt,
    protocol::xproto::EventMask,
    protocol::xproto::MapRequestEvent,
};
use x11rb::protocol::xproto::UnmapNotifyEvent;

use crate::connection::Connection;
use crate::desktop::{Desktop, DesktopMode};
use x11rb::errors::ConnectionError;
use std::os::unix::io::{AsRawFd, RawFd};

pub struct WindowManager<'a> {
    connection: &'a mut Connection<'a>,
    desktops: Vec<Desktop<'a>>,
    active_desktop: usize,
}

impl<'a> WindowManager<'a> {
    /// Create a new WindowManager instance.
    pub fn new(connection: &'a mut Connection<'a>) -> Result<WindowManager<'a>, ReplyOrIdError> {
        let mut wm = Self {
            connection,
            desktops: Vec::new(),
            active_desktop: 0,
        };

        for _ in 0..4 {
            wm.desktops.push(
                Desktop::new(DesktopMode::Tile)
            );
        }

        Ok(wm)
    }

    /// Try to change a root window property to assert another WM is not running.
    pub fn check_if_another_wm_is_running(&self) -> Result<(), Box<dyn Error>> {
        // Create a change and mask event to obtain lock.
        let change = ChangeWindowAttributesAux::default()
            .event_mask(EventMask::SubstructureRedirect | EventMask::SubstructureNotify);

        // Try to take control over the root window.
        match self.connection.dpy.change_window_attributes(self.connection.screen.root, &change)?.check() {
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

    fn on_map_request(&mut self, event: MapRequestEvent) -> Result<(), Box<dyn Error>> {
        // TODO: Store the window into the binary tree and reconfigure the window to match desktop
        // TODO: mode.

        // Temporary map the window directly on the root window instead of the desktop window.
        self.connection.dpy
            .change_window_attributes(
                event.window,
                &ChangeWindowAttributesAux::default()
                    .event_mask(EventMask::FocusChange),
            )?;

        self.connection.dpy.reparent_window(event.window, self.connection.screen.root, 0, 0)?;
        self.connection.dpy.map_window(event.window)?;

        info!("Map window {}.", event.window);
        Ok(())
    }

    fn on_unmap_notify(&self, event: UnmapNotifyEvent) -> Result<(), Box<dyn Error>> {
        // Rebuild the tree.

        info!("Unmap window {}.", event.window);
        Ok(())
    }

    fn on_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<(), Box<dyn Error>> {
        // TODO: Configure a window using element given from the request. We can't configure it
        // TODO: using the binary tree configuration right here because the window is mapped yet.
        self.connection.dpy
            .configure_window(
                event.window,
                &ConfigureWindowAux::default()
                    .x(i32::from(event.x))
                    .y(i32::from(event.y))
                    .height(u32::from(event.height))
                    .width(u32::from(event.width)),
            )?;

        info!("Configured window {}.", event.window);
        Ok(())
    }

    /// Handle an X11 event.
    pub fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        let handle = match event {
            Event::MapRequest(e) => self.on_map_request(e),
            Event::ConfigureRequest(e) => self.on_configure_request(e),
            Event::UnmapNotify(e) => self.on_unmap_notify(e),

            // Handle all other cases.
            _ => {
                debug!("Event not managed : {:?}.", event);
                Ok(())
            }
        };

        if let Err(error) = handle {
            println!("An error occurred for event {:?}: {:?}", event, error);
        }

        Ok(())
    }

    /// Handle a user command through `ibsc`.
    pub fn handle_command(&mut self, command: String) {
        info!("Applying command \"{}\".", command);
    }

    /// Poll an event from X11 server.
    pub fn poll_for_event(&self) -> Result<Option<Event>, ConnectionError> {
        self.connection.dpy.poll_for_event()
    }

    /// Send pending requests to X11 server.
    pub fn flush(&self) -> Result<(), ConnectionError> {
        self.connection.dpy.flush()
    }

    /// Retrieve the connection raw file descriptor.
    pub fn connection_fd(&self) -> RawFd {
        self.connection.dpy.as_raw_fd()
    }
}

impl<'a> Debug for WindowManager<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, desktop) in self.desktops.iter().enumerate() {
            let mut clients_count = 0;
            let mut client = desktop.head;

            while let Some(c) = client {
                clients_count += 1;
                client = match &c.next {
                    Some(c) => Some(&*c),
                    None => None,
                };
            }

            writeln!(f, "{}:{}:{:?}:{}", i, clients_count, desktop.mode, self.active_desktop == i)?;
        }

        Ok(())
    }
}