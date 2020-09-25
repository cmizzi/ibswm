use crate::window_manager::WindowManager;
use ibsc::cli::{Monitor, SubCommand};
use std::error::Error;

pub struct CommandExecutor<'a, 'b: 'a> {
    command: &'a SubCommand,
    wm: &'a mut WindowManager<'b>,
}

impl<'a, 'b: 'a> CommandExecutor<'a, 'b> {
    /// Create a new CommandExecutor instance.
    pub fn new(wm: &'a mut WindowManager<'b>, command: &'a SubCommand) -> Self {
        Self { command, wm }
    }

    /// Execute the command.
    pub fn execute(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("Execute command : {:?}", &self.command);

        match self.command {
            SubCommand::Monitor(command) => self.on_monitor_command(command),
            _ => Err("Undefined command behavior.".into()),
        }
    }

    /// Handle `ibsc monitor` commands.
    fn on_monitor_command(&mut self, command: &Monitor) -> Result<(), Box<dyn Error>> {
        let monitor = self
            .wm
            .monitors
            .find_by_name(&command.monitor)
            .ok_or(format!("cannot find monitor {}", &command.monitor))?;

        // Alias a monitor.
        if let Some(alias) = &command.name {
            monitor.set_alias(alias);
        }

        Ok(())
    }
}
