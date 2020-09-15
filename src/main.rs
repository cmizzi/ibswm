// A very simple reparenting window manager.
// This WM does NOT follow ICCCM!

extern crate x11rb;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::process::exit;

use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;

/// The state of a single window that we manage
#[derive(Debug)]
struct WindowState {
    window: Window,
    frame_window: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl WindowState {
    fn new(window: Window, frame_window: Window, geom: &GetGeometryReply) -> WindowState {
        WindowState {
            window,
            frame_window,
            x: 0,
            y: 0,
            width: 1024,
            height: 864,
        }
    }
}

/// The state of the full WM
#[derive(Debug)]
struct WMState<'a, C: Connection> {
    conn: &'a C,
    screen_num: usize,
    black_gc: Gcontext,
    windows: Vec<WindowState>,
    pending_expose: HashSet<Window>,
    wm_protocols: Atom,
    wm_delete_window: Atom,
    sequences_to_ignore: BinaryHeap<Reverse<u16>>,
}

impl<'a, C: Connection> WMState<'a, C> {
    fn new(conn: &'a C, screen_num: usize) -> Result<WMState<'a, C>, ReplyOrIdError> {
        let black_gc = conn.generate_id()?;
        let wm_protocols = conn.intern_atom(false, b"WM_PROTOCOLS")?;
        let wm_delete_window = conn.intern_atom(false, b"WM_DELETE_WINDOW")?;

        Ok(WMState {
            conn,
            screen_num,
            black_gc,
            windows: Vec::default(),
            pending_expose: HashSet::default(),
            wm_protocols: wm_protocols.reply()?.atom,
            wm_delete_window: wm_delete_window.reply()?.atom,
            sequences_to_ignore: Default::default(),
        })
    }

    /// Scan for already existing windows and manage them
    fn scan_windows(&mut self) -> Result<(), ReplyOrIdError> {
        // Get the already existing top-level windows.
        let screen = &self.conn.setup().roots[self.screen_num];
        let tree_reply = self.conn.query_tree(screen.root)?.reply()?;

        // For each window, request its attributes and geometry *now*
        let mut cookies = Vec::with_capacity(tree_reply.children.len());

        for win in tree_reply.children {
            let attr = self.conn.get_window_attributes(win)?;
            let geom = self.conn.get_geometry(win)?;

            cookies.push((win, attr, geom));
        }

        // Get the replies and manage windows
        for (win, attr, geom) in cookies {
            let (attr, geom) = (attr.reply(), geom.reply());

            if attr.is_err() || geom.is_err() {
                // Just skip this window
                continue;
            }

            let (attr, geom) = (attr.unwrap(), geom.unwrap());

            if !attr.override_redirect && attr.map_state != MapState::Unmapped {
                self.manage_window(win, &geom)?;
            }
        }

        Ok(())
    }

    /// Add a new window that should be managed by the WM
    fn manage_window(&mut self, win: Window, geom: &GetGeometryReply) -> Result<(), ReplyOrIdError> {
        println!("Managing window {:?}", win);
        let screen = &self.conn.setup().roots[self.screen_num];
        assert!(self.find_window_by_id(win).is_none());

        let frame_win = self.conn.generate_id()?;
        let win_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::Exposure
                    | EventMask::SubstructureNotify
                    | EventMask::ButtonPress
                    | EventMask::ButtonRelease
                    | EventMask::PointerMotion
                    | EventMask::EnterWindow,
            )
            .background_pixel(screen.white_pixel);

        self.conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            frame_win,
            screen.root,
            geom.x,
            geom.y,
            geom.width,
            geom.height,
            1,
            WindowClass::InputOutput,
            0,
            &win_aux,
        )?;

        self.conn.grab_server()?;
        self.conn.change_save_set(SetMode::Insert, win)?;
        let cookie = self.conn.reparent_window(win, frame_win, 0, 0)?;

        self.conn.map_window(win)?;
        self.conn.map_window(frame_win)?;
        self.conn.ungrab_server()?;

        self.windows.push(WindowState::new(win, frame_win, geom));

        // Ignore all events caused by reparent_window(). All those events have the sequence number
        // of the reparent_window() request, thus remember its sequence number. The
        // grab_server()/ungrab_server() is done so that the server does not handle other clients
        // in-between, which could cause other events to get the same sequence number.
        self.sequences_to_ignore.push(Reverse(cookie.sequence_number() as u16));

        Ok(())
    }

    /// Do all pending work that was queued while handling some events
    fn refresh(&mut self) -> Result<(), ReplyError> {
        while let Some(&win) = self.pending_expose.iter().next() {
            self.pending_expose.remove(&win);
        }

        Ok(())
    }

    fn find_window_by_id(&self, win: Window) -> Option<&WindowState> {
        self.windows
            .iter()
            .find(|state| state.window == win || state.frame_window == win)
    }

    fn find_window_by_id_mut(&mut self, win: Window) -> Option<&mut WindowState> {
        self.windows
            .iter_mut()
            .find(|state| state.window == win || state.frame_window == win)
    }

    /// Handle the given event
    fn handle_event(&mut self, event: Event) -> Result<(), ReplyOrIdError> {
        let mut should_ignore = false;

        if let Some(seqno) = event.wire_sequence_number() {
            // Check sequences_to_ignore and remove entries with old (=smaller) numbers.
            while let Some(&Reverse(to_ignore)) = self.sequences_to_ignore.peek() {
                // Sequence numbers can wrap around, so we cannot simply check for
                // "to_ignore <= seqno". This is equivalent to "to_ignore - seqno <= 0", which is what we
                // check instead. Since sequence numbers are unsigned, we need a trick: We decide
                // that values from [MAX/2, MAX] count as "<= 0" and the rest doesn't.
                if to_ignore.wrapping_sub(seqno) <= u16::max_value() / 2 {
                    // If the two sequence numbers are equal, this event should be ignored.
                    should_ignore = to_ignore == seqno;
                    break;
                }

                self.sequences_to_ignore.pop();
            }
        }

        println!("Got event {:?}", event);

        if should_ignore {
            println!(" [ignored]");
            return Ok(());
        }

        match event {
            Event::UnmapNotify(event) => self.handle_unmap_notify(event)?,
            Event::ConfigureRequest(event) => self.handle_configure_request(event)?,
            Event::MapRequest(event) => self.handle_map_request(event)?,
            Event::Expose(event) => self.handle_expose(event)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<(), ReplyError> {
        let root = self.conn.setup().roots[self.screen_num].root;
        let conn = self.conn;

        self.windows.retain(|state| {
            if state.window != event.window {
                return true;
            }

            conn.change_save_set(SetMode::Delete, state.window).unwrap();
            conn.reparent_window(state.window, root, state.x, state.y).unwrap();
            conn.destroy_window(state.frame_window).unwrap();

            false
        });

        Ok(())
    }

    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<(), ReplyError> {
        if let Some(state) = self.find_window_by_id_mut(event.window) {
            let _ = state;
            unimplemented!();
        }

        let mut aux = ConfigureWindowAux::default();

        if event.value_mask & u16::from(ConfigWindow::X) != 0 {
            aux = aux.x(i32::from(event.x));
        }

        if event.value_mask & u16::from(ConfigWindow::Y) != 0 {
            aux = aux.y(i32::from(event.y));
        }

        if event.value_mask & u16::from(ConfigWindow::Width) != 0 {
            aux = aux.width(u32::from(event.width));
        }

        if event.value_mask & u16::from(ConfigWindow::Height) != 0 {
            aux = aux.height(u32::from(event.height));
        }

        println!("Configure: {:?}", aux);
        self.conn.configure_window(event.window, &aux)?;

        Ok(())
    }

    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<(), ReplyOrIdError> {
        self.manage_window(
            event.window,
            &self.conn.get_geometry(event.window)?.reply()?,
        )
    }

    fn handle_expose(&mut self, event: ExposeEvent) -> Result<(), ReplyError> {
        self.pending_expose.insert(event.window);
        Ok(())
    }
}

fn become_wm<C: Connection>(conn: &C, screen: &Screen) -> Result<(), ReplyError> {
    // Try to become the window manager. This causes an error if there is already another WM.
    let change = ChangeWindowAttributesAux::default()
        .event_mask(EventMask::SubstructureRedirect | EventMask::SubstructureNotify);

    let res = conn.change_window_attributes(screen.root, &change)?.check();

    match res {
        // Capture only the ACCESS_ERROR message.
        Err(ReplyError::X11Error(error)) if error.error_code() == ACCESS_ERROR => {
            println!("It seems another WM is already running. Error: {:?}", error);
            exit(1);
        },

        // For other error, just show it.
        Err(error) => {
            println!("Error: {:?}", error);
            exit(1);
        },

        // Return by default the response.
        _ => res,
    }
}

fn main() {
    let (conn, screen_num) = x11rb::connect(None).expect("Unable to connect to X server.");
    let screen = &conn.setup().roots[screen_num];

    become_wm(&conn, screen).unwrap();

    let mut wm_state = WMState::new(&conn, screen_num).unwrap();
    wm_state.scan_windows().unwrap();

    loop {
        wm_state.refresh().unwrap();
        conn.flush().unwrap();

        let event = conn.wait_for_event().unwrap();
        let mut event_option = Some(event);

        while let Some(event) = event_option {
            wm_state.handle_event(event).unwrap();
            event_option = conn.poll_for_event().unwrap();
        }
    }
}
