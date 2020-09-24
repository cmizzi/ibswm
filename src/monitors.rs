use crate::desktop::{Desktop, DesktopMode};

pub struct Monitor {
    pub name: String,
    pub alias: Option<String>,
    desktops: Vec<Desktop>,
}

impl Monitor {
    pub fn new(name: String) -> Self {
        let mut monitor = Self {
            name,
            alias: None,
            desktops: Vec::new(),
        };

        monitor.add_desktop(DesktopMode::Tile);
        monitor
    }

    /// Add a new desktop.
    pub fn add_desktop(&mut self, mode: DesktopMode) {
        self.desktops.push(
            Desktop::new(mode)
        );
    }

    /// Alias a monitor.
    pub fn set_alias(&mut self, alias: &str) {
        self.alias = Some(alias.to_string());
    }
}

/// Represents all monitors managed by the window manager.
pub struct Monitors(Vec<Monitor>);

impl Monitors {
    /// Create a list of empty monitors.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a new monitor.
    pub fn add_monitor(&mut self, monitor: Monitor) {
        self.0.push(monitor);
    }

    /// Find a monitor by name or alias.
    pub fn find_by_name(&mut self, name: &str) -> Option<&mut Monitor> {
        for monitor in self.0.iter_mut() {
            if monitor.name == name || (monitor.alias.is_some() && monitor.alias.as_ref().unwrap() == name) {
                return Some(monitor);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::monitors::{Monitors, Monitor};
    use std::borrow::BorrowMut;

    #[test]
    fn a_name_is_correctly_set_when_adding_a_monitor() {
        let mut monitors = Monitors::new();
        monitors.add_monitor(Monitor::new("test".to_string()));

        assert_eq!("test".to_string(), monitors.0[0].name);
        assert_eq!(1, monitors.0.len());
    }

    #[test]
    fn a_monitor_can_be_find_by_name() {
        let mut monitors = Monitors::new();
        monitors.add_monitor(Monitor::new("HDMI-0".to_string()));

        let monitor = monitors.find_by_name("HDMI-0");

        assert_eq!(true, monitor.is_some());
        assert_eq!("HDMI-0".to_string(), monitor.unwrap().name);
        assert_eq!(1, monitors.0.len());
    }

    #[test]
    fn a_monitor_can_be_aliased() {
        let mut monitors = Monitors::new();
        monitors.add_monitor(Monitor::new("HDMI-0".to_string()));

        let monitor = monitors.find_by_name("HDMI-0").unwrap();
        monitor.set_alias("left");

        assert_eq!(true, monitor.alias.is_some());
        assert_eq!("left", monitor.alias.as_ref().unwrap());
        assert_eq!(1, monitors.0.len());
    }
}