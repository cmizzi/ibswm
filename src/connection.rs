use std::collections::HashMap;
use std::error::Error;
use std::str;

#[allow(unused_imports)]
use x11rb::connection::{Connection as _, RequestConnection as _};
use x11rb::connection::Connection as Conn;
use x11rb::protocol::xproto::*;
#[allow(unused_imports)]
use x11rb::wrapper::ConnectionExt as _;

pub struct Connection<'a, C: Conn> {
    pub dpy: &'a C,
    pub screen: Screen,
    atoms: HashMap<String, Atom>,
}

impl<'a, C: Conn> Connection<'a, C> {
    pub fn new(connection: &'a C, screen_key: usize) -> Self {
        let mut c = Self {
            dpy: connection,
            screen: connection.setup().roots[screen_key].clone(),
            atoms: HashMap::new(),
        };

        // Setting up basic ICCCM states.
        c.dpy
            .change_property32(
                PropMode::Replace,
                c.screen.root,
                c.atom(b"_NET_SUPPORTED").unwrap(),
                AtomEnum::ATOM,
                &[
                    c.atom(b"_NET_SUPPORTED").unwrap(),
                    c.atom(b"_NET_WM_STATE").unwrap(),
                    c.atom(b"_NET_ACTIVE_WINDOW").unwrap(),
                    c.atom(b"_NET_WM_STATE_FULLSCREEN").unwrap(),
                ],
            )
            .expect("Cannot add _NET_SUPPORTED property atom.");

        debug!("Applying ICCCM support on main root.");

        // Force applying change right now.
        c.dpy.flush().unwrap();
        c
    }

    pub fn atom<'b>(&mut self, name: &'b [u8]) -> Result<Atom, Box<dyn Error>> {
        let name_string = str::from_utf8(name)?.to_string();
        let dpy = self.dpy;

        let atom = self.atoms.entry(name_string)
            .or_insert_with(|| dpy
                .intern_atom(false, name)
                .map(|cookie| {
                    cookie
                        .reply()
                        .map(|reply| reply.atom)
                        .unwrap()
                })
                .unwrap()
            );

        Ok(*atom)
    }
}
