use ibsc::cli::{SubCommand, Monitor};
use std::os::unix::net::UnixStream;
use std::error::Error;
use std::io::Write;
use crate::window_manager::WindowManager;

pub struct CommandExecutor<'a, 'b: 'a> {
    command: &'a SubCommand,
    wm: &'a mut WindowManager<'b>,
}

impl<'a, 'b: 'a> CommandExecutor<'a, 'b> {
    pub fn new(wm: &'a mut WindowManager<'b>, command: &'a SubCommand) -> Self {
        Self {
            command,
            wm,
        }
    }

    pub fn execute(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("Execute command : {:?}", &self.command);

        match self.command {
            SubCommand::Monitor(command) => self.on_monitor_command(command),
            _ => Err("Undefined command behavior.".into()),
        }
    }

    fn on_monitor_command(&mut self, command: &Monitor) -> Result<(), Box<dyn Error>> {
        let monitor = self.wm.monitors.find_by_name(&command.monitor).ok_or(format!("cannot find monitor {}", &command.monitor))?;

        Ok(())
    }
}