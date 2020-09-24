#[derive(Debug)]
pub enum DesktopMode {
    Tile,
}

pub struct Desktop {
    pub mode: DesktopMode,
}

impl Desktop {
    pub fn new(mode: DesktopMode) -> Self {
        Self {
            mode
        }
    }
}
