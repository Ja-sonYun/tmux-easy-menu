use crate::tmux::positions::{bottom, right};
use serde::{Deserialize, Serialize};
use std::option::Option;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    #[serde(default = "right")]
    pub x: String,

    #[serde(default = "bottom")]
    pub y: String,

    #[serde(default)]
    pub w: Option<String>,

    #[serde(default)]
    pub h: Option<String>,
}

impl Position {
    pub fn new_xy() -> Self {
        Self {
            x: right(),
            y: bottom(),
            w: None,
            h: None,
        }
    }

    pub fn new_xywh() -> Self {
        Self {
            x: right(),
            y: bottom(),
            w: Some("100".to_string()),
            h: Some("100".to_string()),
        }
    }

    pub fn wh(w: i32, h: i32) -> Self {
        Self {
            x: right(),
            y: bottom(),
            w: Some(w.to_string()),
            h: Some(h.to_string()),
        }
    }

    pub fn as_this_arguments(&self) -> Vec<String> {
        let mut arguments: Vec<String> = Vec::new();

        if let Some(w) = &self.w {
            arguments.push("--w".to_string());
            arguments.push(w.to_string());
        }

        if let Some(h) = &self.h {
            arguments.push("--h".to_string());
            arguments.push(h.to_string());
        }

        arguments.push("--x".to_string());
        arguments.push(self.x.to_string());

        arguments.push("--y".to_string());
        arguments.push(self.y.to_string());

        arguments
    }
}
