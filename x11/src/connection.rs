use x11rb::connection::Connection as Conn;
use crate::window::WindowActions;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt as _;
use std::error::Error;
use x11rb::errors::ConnectionError;
use std::collections::HashMap;
use std::str;

pub struct Connection<'a, C: Conn> {
    pub conn: &'a C,
    pub screen: Screen,
    atoms: HashMap<String, Atom>,
}

impl<'a, C: Conn> Connection<'a, C> {
    pub fn new(connection: &'a C, screen_key: usize) -> Self {
        Self {
            conn: connection,
            screen: connection.setup().roots[screen_key].clone(),
            atoms: HashMap::new(),
        }
    }

    pub fn number_of_screens(&self) -> u8 {
        self.conn.setup().roots_len()
    }

    pub fn gracefully_destroy_window(&mut self, window: Window) -> Result<(), Box<dyn Error>> {
        let protocols = self.get_property(window, b"WM_PROTOCOL")?;
        let wm_delete_window = self.atom(b"WM_DELETE_WINDOW")?;

        for protocol in protocols.value.iter() {
            if *protocol != wm_delete_window as u8 {
                continue;
            }

            println!("Gracefully destroy the window.");

            self.conn
                .send_event(
                    false,
                    window,
                    EventMask::NoEvent,
                    ClientMessageEvent {
                        response_type: 0,
                        format: 32,
                        sequence: 0,
                        window,
                        type_: wm_delete_window,
                        data: ClientMessageData::from([
                            wm_delete_window,
                            x11rb::CURRENT_TIME,
                            0,
                            0,
                            0
                        ])
                    }
                )?;

            return Ok(());
        }

        // If the window doesn't follow ICCCM, just destroy it.
        println!("Destroy the window.");
        self.conn.destroy_window(window);

        Ok(())
    }

    pub fn get_property<'b>(&mut self, window: Window, name: &'b [u8]) -> Result<GetPropertyReply, Box<dyn Error>> {
        let reply = self.conn
            .get_property(false, window, self.atom(name)?, GetPropertyType::Any, 0, 64)?
            .reply()?;

        Ok(reply)
    }

    pub fn atom<'b>(&mut self, name: &'b [u8]) -> Result<Atom, Box<dyn Error>> {
        let name_string = str::from_utf8(name)?.to_string();

        let atom = self.atoms.entry(name_string)
            .or_insert(
                self.conn
                    .intern_atom(false, name)
                    .map(|cookie| {
                        cookie
                            .reply()
                            .map(|reply| reply.atom)
                            .unwrap()
                    })
                    .unwrap()
            );

        Ok(atom.clone())
    }
}
