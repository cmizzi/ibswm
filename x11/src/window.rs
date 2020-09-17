use x11rb::protocol::xproto::Window;

pub trait WindowActions {
    fn gracefully_destroy_window(window: Window) {
    }
}