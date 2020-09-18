use x11::x11rb::protocol::xproto::Window;

pub struct Client<'a> {
    pub next: Option<&'a Client<'a>>,
    pub is_urgent: bool,
    pub is_float: bool,
    pub is_transient: bool,
    pub window: Window,
}