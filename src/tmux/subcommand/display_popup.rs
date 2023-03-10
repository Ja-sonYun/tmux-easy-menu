use std::process::Child;

use crate::show::construct_position::Position;
use crate::tmux::Tmux;
use anyhow::Result;

static DISPLAY_POPUP: &str = "display-popup";

impl Tmux {
    fn construct_border_arguments(border: &str) -> Vec<String> {
        vec!["-b".to_string(), border.to_string()]
    }

    pub fn display_popup(&self, command: String, position: &Position, exit: bool) -> Result<Child> {
        let mut arguments = vec![DISPLAY_POPUP.to_string()];

        arguments.append(&mut Self::construct_border_arguments("double"));
        arguments.append(&mut Self::construct_position_arguments(position));
        if exit {
            arguments.push("-E".to_string());
        }
        arguments.push(command);

        let child = self._run(arguments, false)?;

        Ok(child)
    }
}
