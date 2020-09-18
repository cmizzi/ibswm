use std::error::Error;
use std::fmt::{Debug, Formatter};

use x11::connection::Connection;
use x11::x11rb::{
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

use crate::desktop::{Desktop, DesktopMode};

pub struct WindowManager<'a, C: X11Connection> {
    connection: &'a mut Connection<'a, C>,
    desktops: Vec<Desktop<'a>>,
    active_desktop: usize,
}

impl<'a, C: X11Connection> WindowManager<'a, C> {
    /// Create a new WindowManager instance.
    pub fn new(connection: &'a mut Connection<'a, C>) -> Result<WindowManager<'a, C>, ReplyOrIdError> {
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

    fn on_map_request(&mut self, _event: MapRequestEvent) -> Result<(), Box<dyn Error>> {
        println!("{:?}", self);

        Ok(())
    }

    fn on_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<(), Box<dyn Error>> {
        self.connection.dpy
            .configure_window(
                event.window,
                &ConfigureWindowAux::default()
                    .x(i32::from(event.x))
                    .y(i32::from(event.y))
                    .height(u32::from(event.height))
                    .width(u32::from(event.width)),
            )?;

        println!("Configured window {}.", event.window);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("{:?}", self);

        loop {
            self.connection.dpy.flush()?;
            let mut event = Some(self.connection.dpy.wait_for_event()?);

            while let Some(e) = event {
                let handle = match e {
                    Event::MapRequest(e) => self.on_map_request(e),
                    Event::ConfigureRequest(e) => self.on_configure_request(e),

                    // Handle all other cases.
                    _ => {
                        println!("Event not managed : {:?}.", e);
                        Ok(())
                    },
                };

                if let Err(error) = handle {
                    println!("An error occured for event {:?}: {:?}", e, error);
                }

                event = self.connection.dpy.poll_for_event()?;
            }
        }
    }
}

impl<'a, C: X11Connection> Debug for WindowManager<'a, C> {
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