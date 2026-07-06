pub mod positions;
pub mod subcommand;

use std::process::Child;

use crate::shell::spawn_binary;
use crate::show::construct_position::Position;
use anyhow::Result;

pub struct Tmux {
    binary: String,
}

impl Tmux {
    fn _run(&self, arguments: Vec<String>) -> Result<Child> {
        spawn_binary(self.binary.clone(), arguments)
    }

    fn construct_border_arguments(border: &str) -> Vec<String> {
        vec!["-b".to_string(), border.to_string()]
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
            binary: "tmux".to_string(),
        }
    }
}
