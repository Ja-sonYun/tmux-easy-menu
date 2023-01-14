use crate::show::construct_position::Position;
use crate::tmux::Tmux;
use anyhow::Result;

static DISPLAY_POPUP: &str = "display-popup";

impl Tmux {
    pub fn display_popup(
        &self,
        command: String,
        position: &Position,
        exit: bool,
        non_block: bool,
    ) -> Result<()> {
        let mut arguments = vec![DISPLAY_POPUP.to_string()];

        arguments.append(&mut Self::construct_position_arguments(position));
        if exit {
            arguments.push("-E".to_string());
        }
        arguments.push(command);

        self._run(arguments, non_block)
    }
}
