use crate::shell::{run_command, shell_quote};

use crate::show::construct_position::Position;
use crate::show::this::run_this_with;
use anyhow::{bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MenuType {
    Menu {
        name: String,

        #[serde(default)]
        shortcut: String,

        #[serde(default)]
        command: Option<String>,

        #[serde(default)]
        next_menu: Option<String>,

        #[serde(default = "default_true")]
        close_after_command: bool,

        #[serde(default)]
        session: bool,

        #[serde(default)]
        session_name: Option<String>,

        #[serde(default)]
        key_table: Option<String>,

        #[serde(default)]
        session_on_dir: bool,

        #[serde(default)]
        run_on_git_root: bool,

        #[serde(default)]
        background: bool,

        #[serde(default)]
        as_floating_pane: bool,

        #[serde(default)]
        inputs: Vec<String>,

        #[serde(default = "Position::new_xywh")]
        position: Position,

        #[serde(default)]
        border: Option<String>,

        #[serde(default)]
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
                if !name.contains("$(") {
                    return;
                }

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

    pub fn get_execute_command(
        &self,
        path: &PathBuf,
        on_dir: &PathBuf,
        cli_x: Option<&str>,
        cli_y: Option<&str>,
    ) -> Result<String> {
        match self {
            MenuType::NoDim { .. } | MenuType::Seperate { .. } => {
                bail!("This menu type should be menu")
            }
            MenuType::Menu {
                command,
                next_menu,
                close_after_command,
                background,
                as_floating_pane,
                position,
                session,
                session_name,
                key_table,
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
                    let next_menu_path = next_menu_path
                        .to_str()
                        .ok_or_else(|| anyhow::anyhow!("next menu path is not valid UTF-8"))?;
                    wrapped_command.push(next_menu_path.to_string());

                    wrapped_command.push("--working_dir".to_string());
                    let on_dir = on_dir.to_str().ok_or_else(|| {
                        anyhow::anyhow!("working directory path is not valid UTF-8")
                    })?;
                    wrapped_command.push(on_dir.to_string());

                    if let Some(x) = cli_x {
                        wrapped_command.push("--x".to_string());
                        wrapped_command.push(x.to_string());
                    }

                    if let Some(y) = cli_y {
                        wrapped_command.push("--y".to_string());
                        wrapped_command.push(y.to_string());
                    }
                } else if let Some(command) = command {
                    let working_dir_path = if *run_on_git_root {
                        Self::find_git_root(on_dir).unwrap_or_else(|| on_dir.clone())
                    } else {
                        on_dir.clone()
                    };
                    let working_dir = working_dir_path.to_str().ok_or_else(|| {
                        anyhow::anyhow!("working directory path is not valid UTF-8")
                    })?;
                    let quoted_working_dir = shell_quote(working_dir);

                    let command = command.replace("%%PWD", working_dir);

                    if *background {
                        if environment.is_empty() {
                            return Ok(format!("cd {} && {}", quoted_working_dir, command));
                        } else {
                            let env_setup = environment
                                .iter()
                                .map(|(k, v)| {
                                    format!(
                                        "tmux set-environment {} {} && export {}={}",
                                        k,
                                        shell_quote(v),
                                        k,
                                        shell_quote(v)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" && ");
                            return Ok(format!(
                                "{} && cd {} && {}",
                                env_setup, quoted_working_dir, command
                            ));
                        }
                    }
                    wrapped_command.push("popup".to_string());

                    if *as_floating_pane {
                        wrapped_command.push("--as_floating_pane".to_string());
                    }

                    if let Some(key_table) = key_table {
                        wrapped_command.push("--key_table".to_string());
                        wrapped_command.push(key_table.to_string());
                    }

                    wrapped_command.push("--working_dir".to_string());
                    wrapped_command.push(working_dir.to_string());

                    wrapped_command.push("--cmd".to_string());
                    if *session {
                        let session_part = if let Some(session_name) = session_name {
                            session_name.to_string()
                        } else {
                            format!("session_{}", command.replace(" ", "_"))
                        };

                        let base_session_name = if *session_on_dir {
                            let dir_for_session = if *run_on_git_root {
                                working_dir_path.clone()
                            } else {
                                on_dir.clone()
                            };
                            let full_path = dir_for_session
                                .to_str()
                                .unwrap_or("unknown")
                                .replace(['/', ' ', '.', ':'], "_");
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
                        let quoted_session = shell_quote(&_session_name);
                        let set_key_table = key_table
                            .as_ref()
                            .map(|key_table| {
                                format!(
                                    "tmux set-option -t {quoted_session} key-table {} 2>/dev/null && ",
                                    shell_quote(key_table)
                                )
                            })
                            .unwrap_or_default();
                        let env_flags = if environment.is_empty() {
                            String::new()
                        } else {
                            environment
                                .iter()
                                .map(|(k, v)| format!("-e {}={}", k, shell_quote(v)))
                                .collect::<Vec<_>>()
                                .join(" ")
                                + " "
                        };
                        let attach = if *as_floating_pane {
                            "TMUX= tmux -S \"$(tmux display-message -p '#{socket_path}')\" attach"
                        } else {
                            "tmux attach"
                        };

                        wrapped_command.push(format!(
                            "{set_key_table}{attach} -t {session} 2>/dev/null || \
                            (cd {working_dir} && tmux new-session -d -s {session} {env_flags}{command} 2>/dev/null && \
                            tmux set-option -t {session} status off 2>/dev/null && \
                            {set_key_table}\
                            {attach} -t {session})",
                            session = quoted_session,
                            working_dir = quoted_working_dir,
                            command = shell_quote(&command),
                            env_flags = env_flags
                        ));
                        wrapped_command.push("--session_name".to_string());
                        wrapped_command.push(_session_name);
                    } else {
                        if environment.is_empty() {
                            wrapped_command
                                .push(format!("cd {} && {}", quoted_working_dir, command));
                        } else {
                            let env_setup = environment
                                .iter()
                                .map(|(k, v)| {
                                    format!(
                                        "tmux set-environment {} {} && export {}={}",
                                        k,
                                        shell_quote(v),
                                        k,
                                        shell_quote(v)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" && ");
                            wrapped_command.push(format!(
                                "{} && cd {} && {}",
                                env_setup, quoted_working_dir, command
                            ));
                        }
                    }

                    let mut effective_position = position.clone();
                    if let Some(x) = cli_x {
                        effective_position.x = x.to_string();
                    }
                    if let Some(y) = cli_y {
                        effective_position.y = y.to_string();
                    }
                    wrapped_command.extend(effective_position.as_this_arguments());

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
        let mut dir = start_dir.clone();

        loop {
            let git = dir.join(".git");
            if git.is_dir() || git.is_file() {
                return Some(dir);
            }

            if !dir.pop() {
                return None;
            }
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

    #[serde(skip)]
    pub cli_x: Option<String>,

    #[serde(skip)]
    pub cli_y: Option<String>,

    #[serde(default = "Position::new_xy")]
    pub position: Position,

    pub title: String,
    pub items: Vec<MenuType>,

    #[serde(default = "default_border")]
    pub border: String,
}

impl Menus {
    pub fn load(path: PathBuf, cwd: PathBuf) -> Result<Menus> {
        Self::load_with_eval(path, cwd, true)
    }

    pub fn load_for_select(path: PathBuf, cwd: PathBuf) -> Result<Menus> {
        Self::load_with_eval(path, cwd, false)
    }

    fn load_with_eval(path: PathBuf, cwd: PathBuf, eval_names: bool) -> Result<Menus> {
        let file = File::open(&path)?;
        let mut menus: Menus = serde_yaml::from_reader(file)?;
        menus.conf_path = path;
        menus.cwd = cwd;
        menus.cli_x = None;
        menus.cli_y = None;

        for menu in &mut menus.items {
            if eval_names {
                menu.eval_name();
            }

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

#[cfg(test)]
mod tests {
    use super::MenuType;
    use crate::show::construct_position::Position;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tmux-menu-{name}-{id}"))
    }

    #[test]
    fn find_git_root_walks_parents() {
        let root = test_dir("git-root");
        let nested = root.join("a/b/c");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(&nested).unwrap();

        assert_eq!(MenuType::find_git_root(&nested), Some(root.clone()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn find_git_root_accepts_git_file() {
        let root = test_dir("git-file");
        let nested = root.join("a");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join(".git"), "gitdir: ../real").unwrap();

        assert_eq!(MenuType::find_git_root(&nested), Some(root.clone()));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn find_git_root_returns_none_outside_repo() {
        let root = test_dir("no-git");
        fs::create_dir_all(&root).unwrap();

        assert_eq!(MenuType::find_git_root(&root), None);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn floating_pane_is_opt_in() {
        let menu: MenuType = serde_yaml::from_str("Menu:\n  name: test\n  command: ':'").unwrap();

        match menu {
            MenuType::Menu {
                as_floating_pane, ..
            } => assert!(!as_floating_pane),
            _ => panic!("expected menu"),
        }
    }

    #[test]
    fn session_name_is_quoted_in_shell_command() {
        let root = test_dir("quoted-session");
        fs::create_dir_all(&root).unwrap();

        let menu = MenuType::Menu {
            name: "bad".to_string(),
            shortcut: String::new(),
            command: Some(":".to_string()),
            next_menu: None,
            close_after_command: true,
            session: true,
            session_name: Some("bad; echo pwn".to_string()),
            key_table: Some("popup; echo pwn".to_string()),
            session_on_dir: false,
            run_on_git_root: false,
            background: false,
            as_floating_pane: true,
            inputs: Vec::new(),
            position: Position::new_xywh(),
            border: None,
            environment: HashMap::new(),
        };

        let command = menu
            .get_execute_command(&PathBuf::from("menu.yaml"), &root, None, None)
            .unwrap();
        let args = shlex::split(&command).unwrap();
        let cmd_index = args.iter().position(|arg| arg == "--cmd").unwrap();
        let popup_command = &args[cmd_index + 1];

        assert!(popup_command.contains("TMUX= tmux -S"));
        assert!(popup_command.contains("attach -t '_popup_"));
        assert!(popup_command.contains("bad; echo pwn'"));
        assert!(popup_command.contains("key-table 'popup; echo pwn'"));
        let session_index = args.iter().position(|arg| arg == "--session_name").unwrap();
        assert!(args[session_index + 1].starts_with("_popup_"));
        assert!(args.iter().any(|arg| arg == "--as_floating_pane"));

        fs::remove_dir_all(root).unwrap();
    }
}
