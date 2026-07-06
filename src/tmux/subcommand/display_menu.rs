use std::env::current_exe;
use std::path::PathBuf;
use std::process::Child;

use crate::show::construct_menu::{MenuType, Menus};
use crate::show::this::run_this_binary_with;
use crate::tmux::Tmux;
use anyhow::Result;

static DISPLAY_MENU: &str = "display-menu";

impl Tmux {
    fn tmux_single_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }

    fn tmux_format_escape(value: &str) -> String {
        value.replace('#', "##")
    }

    fn run_shell_action(command: &str) -> String {
        Self::tmux_format_escape(&format!(
            "run-shell -b -- {}",
            Self::tmux_single_quote(command)
        ))
    }

    fn construct_menu_arguments(
        menu_items: &[MenuType],
        prev_path: &PathBuf,
        cwd: &PathBuf,
        cli_x: &Option<String>,
        cli_y: &Option<String>,
    ) -> Result<Vec<String>> {
        let cli_x = cli_x.as_deref();
        let cli_y = cli_y.as_deref();
        let this_binary = current_exe()?;
        let this_binary = this_binary
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("current executable path is not valid UTF-8"))?;
        let prev_path = prev_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("menu path is not valid UTF-8"))?;
        let cwd_str = cwd
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("working directory path is not valid UTF-8"))?;
        let mut arguments = Vec::new();

        for (index, menu) in menu_items.iter().enumerate() {
            match menu {
                MenuType::Menu { name, shortcut, .. } => {
                    let mut command = vec![
                        "select".to_string(),
                        "--menu".to_string(),
                        prev_path.to_string(),
                        "--index".to_string(),
                        index.to_string(),
                        "--working_dir".to_string(),
                        cwd_str.to_string(),
                    ];

                    if let Some(x) = cli_x {
                        command.push("--x".to_string());
                        command.push(x.to_string());
                    }

                    if let Some(y) = cli_y {
                        command.push("--y".to_string());
                        command.push(y.to_string());
                    }

                    let command = run_this_binary_with(cwd, this_binary, command)?;
                    arguments.push(name.clone());
                    arguments.push(shortcut.clone());
                    arguments.push(Self::run_shell_action(&command));
                }
                MenuType::NoDim { name } => {
                    arguments.push(format!("-#[nodim]{}", name));
                    arguments.push("".to_string());
                    arguments.push("".to_string());
                }
                MenuType::Seperate {} => arguments.push("".to_string()),
            }
        }

        Ok(arguments)
    }

    fn construct_title_arguments(title: &str) -> Vec<String> {
        vec![
            "-T".to_string(),
            format!("{}{}", "#[align=centre fg=yellow]", title.to_string()),
        ]
    }

    pub fn display_menu(&self, menu: &Menus, verbose: &u8) -> Result<Child> {
        let mut arguments = vec![DISPLAY_MENU.to_string()];
        if verbose > &1 {
            println!("Displaying: {:?}", menu);
        }
        arguments.append(&mut Self::construct_position_arguments(&menu.position));
        arguments.append(&mut Self::construct_title_arguments(&menu.title));
        arguments.append(&mut Self::construct_border_arguments(&menu.border));
        arguments.push("".to_string());
        arguments.append(&mut Self::construct_menu_arguments(
            &menu.items,
            &menu.conf_path,
            &menu.cwd,
            &menu.cli_x,
            &menu.cli_y,
        )?);
        if verbose > &0 {
            println!("Running: {}", arguments.join(" "));
        }

        self._run(arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::Tmux;

    #[test]
    fn run_shell_action_does_not_use_base64() {
        let action = Tmux::run_shell_action("printf '%s' \"a b;c|d#e\"");

        assert!(action.starts_with("run-shell -b -- "));
        assert!(!action.contains("base64"));
        assert!(!action.contains("base64 -d"));
        assert!(action.contains("##e"));
    }

    #[test]
    fn tmux_single_quote_handles_single_quotes() {
        assert_eq!(Tmux::tmux_single_quote("a'b"), "'a'\\''b'");
    }
}
