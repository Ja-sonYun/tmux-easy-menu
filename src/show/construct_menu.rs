use crate::shell::run_command;

use crate::show::construct_position::Position;
use crate::show::this::run_this_with;
use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

fn default_none() -> Option<String> {
    None
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_vec() -> Vec<String> {
    Vec::new()
}

fn default_empty_string() -> String {
    "".to_string()
}

fn default_hashmap() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MenuType {
    Menu {
        name: String,

        #[serde(default = "default_empty_string")]
        shortcut: String,

        #[serde(default = "default_none")]
        command: Option<String>,

        #[serde(default = "default_none")]
        next_menu: Option<String>,

        #[serde(default = "default_true")]
        close_after_command: bool,

        #[serde(default = "default_false")]
        session: bool,

        #[serde(default = "default_none")]
        session_name: Option<String>,

        #[serde(default = "default_false")]
        session_on_dir: bool,

        #[serde(default = "default_false")]
        run_on_git_root: bool,

        #[serde(default = "default_false")]
        background: bool,

        #[serde(default = "default_vec")]
        inputs: Vec<String>,

        #[serde(default = "Position::new_xywh")]
        position: Position,

        #[serde(default = "default_none")]
        border: Option<String>,

        #[serde(default = "default_hashmap")]
        environment: HashMap<String, String>,
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
                    let output = run_command(command.to_string()).expect("Failed to run command");

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

    pub fn get_execute_command(&self, path: &PathBuf, on_dir: &PathBuf) -> Result<String> {
        match self {
            MenuType::NoDim { .. } | MenuType::Seperate { .. } => {
                bail!("This menu type should be menu")
            }
            MenuType::Menu {
                command,
                next_menu,
                close_after_command,
                background,
                position,
                session,
                session_name,
                session_on_dir,
                run_on_git_root,
                border,
                inputs,
                environment,
                ..
            } => {
                let mut wrapped_command: Vec<String> = Vec::new();

                if let Some(next_menu) = next_menu {
                    wrapped_command.push("show".to_string());
                    wrapped_command.push("--menu".to_string());
                    let next_menu_path = PathBuf::from(next_menu);

                    let prev_parent_path = path.parent().unwrap();
                    let next_menu_path = canonicalize(prev_parent_path.join(next_menu_path))?;

                    if !next_menu_path.exists() {
                        bail!("Next menu path does not exist: {:?}", next_menu_path);
                    }
                    wrapped_command.push(next_menu_path.to_str().unwrap().to_string());

                    wrapped_command.push("--working_dir".to_string());
                    wrapped_command.push(on_dir.to_str().unwrap().to_string());
                } else if let Some(command) = command {
                    let working_dir = if *run_on_git_root {
                        Self::find_git_root(on_dir).unwrap_or_else(|| on_dir.clone())
                    } else {
                        on_dir.clone()
                    };

                    // Replace %%PWD with working directory, and escape double quotes
                    let command = command
                        .replace("\"", "\\\"")
                        .replace("%%PWD", working_dir.to_str().unwrap());

                    if *background {
                        // For background commands, set environment variables before running
                        if environment.is_empty() {
                            return Ok(format!(
                                "cd {} && {}",
                                working_dir.to_str().unwrap(),
                                command
                            ));
                        } else {
                            // Set tmux environment variables AND export them for background commands
                            let env_setup = environment
                                .iter()
                                .map(|(k, v)| {
                                    format!(
                                        "tmux set-environment {} '{}' && export {}='{}'",
                                        k, v, k, v
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" && ");
                            return Ok(format!(
                                "{} && cd {} && {}",
                                env_setup,
                                working_dir.to_str().unwrap(),
                                command
                            ));
                        }
                    }
                    wrapped_command.push("popup".to_string());

                    wrapped_command.push("--working_dir".to_string());
                    wrapped_command.push(working_dir.to_str().unwrap().to_string());

                    wrapped_command.push("--cmd".to_string());
                    // wrapped to move current directory before run command
                    if *session {
                        let session_part = if let Some(session_name) = session_name {
                            session_name.to_string()
                        } else {
                            format!("session_{}", command.replace(" ", "_"))
                        };

                        let base_session_name = if *session_on_dir {
                            let dir_for_session = if *run_on_git_root {
                                working_dir.clone()
                            } else {
                                on_dir.clone()
                            };
                            let full_path = dir_for_session
                                .to_str()
                                .unwrap_or("unknown")
                                .replace("/", "_")
                                .replace(" ", "_");
                            format!("{}_{}", session_part, full_path)
                        } else {
                            session_part
                        };

                        let final_session_name = if *run_on_git_root {
                            format!("git_root_{}", base_session_name)
                        } else {
                            base_session_name
                        };

                        let mut hasher = DefaultHasher::new();
                        final_session_name.hash(&mut hasher);
                        let hash_prefix = format!("{:x}", hasher.finish() % 0xFFFF);

                        let _session_name =
                            format!("_popup_{}_{}", hash_prefix, final_session_name);
                        let encoded_cmd = STANDARD.encode(&command);

                        // Build environment flags for new-session
                        let env_flags = if environment.is_empty() {
                            String::new()
                        } else {
                            environment
                                .iter()
                                .map(|(k, v)| format!("-e {}='{}'", k, v))
                                .collect::<Vec<_>>()
                                .join(" ")
                                + " "
                        };

                        wrapped_command.push(format!(
                            "tmux attach -t {session} 2>/dev/null || \
                            (cd {working_dir} && tmux new-session -d -s {session} {env_flags}\\\"$(echo {encoded_cmd} | base64 -d)\\\" 2>/dev/null && \
                            tmux set-option -t {session} status off 2>/dev/null && \
                            tmux attach -t {session})",
                            session = _session_name,
                            working_dir = working_dir.to_str().unwrap(),
                            encoded_cmd = encoded_cmd,
                            env_flags = env_flags
                        ));
                    } else {
                        // For regular popup commands, set environment variables before running
                        if environment.is_empty() {
                            wrapped_command.push(format!(
                                "cd {} && {}",
                                working_dir.to_str().unwrap(),
                                command
                            ));
                        } else {
                            // Set tmux environment variables AND export them for regular commands
                            let env_setup = environment
                                .iter()
                                .map(|(k, v)| {
                                    format!(
                                        "tmux set-environment {} '{}' && export {}='{}'",
                                        k, v, k, v
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" && ");
                            wrapped_command.push(format!(
                                "{} && cd {} && {}",
                                env_setup,
                                working_dir.to_str().unwrap(),
                                command
                            ));
                        }
                    }

                    wrapped_command.extend(position.as_this_arguments());

                    wrapped_command.push("--border".to_string());
                    if let Some(border) = border {
                        wrapped_command.push(border.to_string());
                    } else {
                        wrapped_command.push("simple".to_string());
                    }

                    if *close_after_command {
                        wrapped_command.push("-E".to_string());
                    }

                    if !inputs.is_empty() {
                        wrapped_command.push("--key".to_string());
                        wrapped_command.extend(inputs.clone());
                    }
                }

                run_this_with(on_dir, wrapped_command)
            }
        }
    }

    fn find_git_root(start_dir: &PathBuf) -> Option<PathBuf> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(start_dir)
            .output()
            .ok()?;

        if output.status.success() {
            let git_root = String::from_utf8(output.stdout).ok()?;
            Some(PathBuf::from(git_root.trim()))
        } else {
            None
        }
    }
}

fn default_border() -> String {
    "single".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menus {
    #[serde(skip)]
    pub conf_path: PathBuf,

    #[serde(skip)]
    pub cwd: PathBuf,

    #[serde(default = "Position::new_xy")]
    pub position: Position,

    pub title: String,
    pub items: Vec<MenuType>,

    #[serde(default = "default_border")]
    pub border: String,
}

impl Menus {
    pub fn load(path: PathBuf, cwd: PathBuf) -> Result<Menus> {
        let file = File::open(&path)?;
        let mut menus: Menus = serde_yaml::from_reader(file)?;
        menus.conf_path = path;
        menus.cwd = cwd;

        for menu in &mut menus.items {
            menu.eval_name();

            // Set default border
            match menu {
                MenuType::Menu { border, .. } => {
                    if border.is_none() {
                        *border = Some(menus.border.clone());
                    }
                }
                _ => {}
            }
        }

        Ok(menus)
    }
}
