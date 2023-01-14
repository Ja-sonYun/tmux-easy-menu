use crate::shell::run_command;
use std::env::current_dir;

use crate::show::construct_position::Position;
use crate::show::this::run_this_with;
use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

fn default_none() -> Option<String> {
    None
}

fn default_true() -> bool {
    true
}

fn default_vec() -> Vec<String> {
    Vec::new()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MenuType {
    Menu {
        name: String,
        shortcut: String,

        #[serde(default = "default_none")]
        command: Option<String>,

        #[serde(default = "default_none")]
        next_menu: Option<String>,

        #[serde(default = "default_true")]
        close_after_command: bool,

        #[serde(default = "default_vec")]
        inputs: Vec<String>,

        #[serde(default = "Position::new_xywh")]
        position: Position,
    },
    NoDim {
        name: String,
    },
    Seperate {},
}

impl MenuType {
    fn set_name(&mut self, name: String) {
        match self {
            MenuType::Menu { name: n, .. } | MenuType::NoDim { name: n, .. } => *n = name,
            MenuType::Seperate {} => {}
        }
    }

    fn eval_name(&mut self) {
        match &self {
            MenuType::Menu { name, .. } | MenuType::NoDim { name, .. } => {
                // regex that variables that wrapped in $()
                let re = Regex::new(r"\$\((.*?)\)").unwrap();
                let mut new_name = name.to_string();

                for cap in re.captures_iter(name) {
                    let command = cap.get(1).unwrap().as_str();
                    let output = run_command(command.to_string());

                    let idx = new_name.find(command).unwrap();
                    let start_idx = idx - 2; // -2 for the $(
                    let end_idx = idx + command.len() + 1; // +1 for the )

                    new_name.replace_range(start_idx..end_idx, &output);
                }
                self.set_name(new_name);
            }
            MenuType::Seperate {} => {}
        }
    }

    pub fn get_execute_command(&self) -> Result<String> {
        match self {
            MenuType::NoDim { .. } | MenuType::Seperate { .. } => {
                bail!("This menu type should be menu")
            }
            MenuType::Menu {
                command,
                next_menu,
                close_after_command,
                position,
                inputs,
                ..
            } => {
                let mut wrapped_command: Vec<String> = Vec::new();

                if let Some(next_menu) = next_menu {
                    wrapped_command.push("show".to_string());
                    wrapped_command.push("--menu".to_string());
                    let next_menu_path = PathBuf::from(next_menu);

                    if !next_menu_path.exists() {
                        bail!("Menu file does not exist: {}", next_menu);
                    }

                    wrapped_command.push(next_menu.to_string());
                } else if let Some(command) = command {
                    wrapped_command.push("popup".to_string());

                    wrapped_command.push("--cmd".to_string());
                    // wrapped to move current directory before run command
                    wrapped_command.push(format!(
                        "cd {} && {}",
                        current_dir()?.display(),
                        command
                    ));

                    wrapped_command.extend(position.as_this_arguments());

                    if *close_after_command {
                        wrapped_command.push("-E".to_string());
                    }

                    if !inputs.is_empty() {
                        wrapped_command.push("--key".to_string());
                        wrapped_command.extend(inputs.clone());
                    }
                }

                run_this_with(wrapped_command)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menus {
    #[serde(skip)]
    menu_path: PathBuf,

    #[serde(default = "Position::new_xy")]
    pub position: Position,

    pub title: String,
    pub items: Vec<MenuType>,
}

impl Menus {
    pub fn load(path: PathBuf) -> Result<Menus> {
        let file = File::open(&path)?;
        let mut menus: Menus = serde_yaml::from_reader(file)?;
        menus.menu_path = path;

        for menu in &mut menus.items {
            menu.eval_name();
        }

        Ok(menus)
    }
}
