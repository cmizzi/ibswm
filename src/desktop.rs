use crate::client::Client;

#[derive(Debug)]
pub enum DesktopMode {
    Tile,
}

pub struct Desktop<'a> {
    pub mode: DesktopMode,
    pub client: Option<&'a Client<'a>>,
    pub head: Option<&'a Client<'a>>,
    pub current: Option<&'a Client<'a>>,
    pub prev: Option<&'a Client<'a>>,
}

impl<'a> Desktop<'a> {
    pub fn new(mode: DesktopMode) -> Self {
        Self {
            mode,
            client: None,
            head: None,
            current: None,
            prev: None,
        }
    }
}