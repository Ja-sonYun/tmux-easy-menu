pub mod positions;
pub mod subcommand;

use crate::shell::{run_command, spawn_binary};
use crate::show::construct_position::Position;
use anyhow::Result;

pub struct Tmux {
    binary: String,
}

impl Tmux {
    fn _get_tmux_binary() -> Result<String> {
        let output = run_command("which tmux".to_string());
        Ok(output)
    }

    fn _run(&self, arguments: Vec<String>, non_block: bool) -> Result<()> {
        spawn_binary(self.binary.clone(), arguments, non_block)?;

        Ok(())
    }

    fn construct_position_arguments(position: &Position) -> Vec<String> {
        if let (Some(w), Some(h)) = (position.w.clone(), position.h.clone()) {
            vec![
                "-x".to_string(),
                position.x.to_string(),
                "-y".to_string(),
                position.y.to_string(),
                "-w".to_string(),
                w.to_string(),
                "-h".to_string(),
                h.to_string(),
            ]
        } else {
            vec![
                "-x".to_string(),
                position.x.to_string(),
                "-y".to_string(),
                position.y.to_string(),
            ]
        }
    }

    pub fn new() -> Self {
        Self {
            binary: Self::_get_tmux_binary().unwrap(),
        }
    }
}
