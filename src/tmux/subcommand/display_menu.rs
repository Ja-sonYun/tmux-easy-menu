use std::path::PathBuf;
use std::process::Child;

use crate::show::construct_menu::{MenuType, Menus};
use crate::tmux::Tmux;
use anyhow::Result;
use base64::Engine as _;

static DISPLAY_MENU: &str = "display-menu";

impl Tmux {
    fn construct_menu_arguments(
        menu_items: &Vec<MenuType>,
        prev_path: &PathBuf,
        cwd: &PathBuf,
        cli_x: &Option<String>,
        cli_y: &Option<String>,
    ) -> Vec<String> {
        let cli_x = cli_x.as_deref();
        let cli_y = cli_y.as_deref();
        menu_items
            .iter()
            .map(|menu| match menu {
                MenuType::Menu { name, shortcut, .. } => {
                    let command = menu
                        .get_execute_command(prev_path, cwd, cli_x, cli_y)
                        .unwrap();
                    // Base64 encode the command to avoid quote escaping issues
                    let encoded_command =
                        base64::engine::general_purpose::STANDARD.encode(&command);
                    vec![
                        name.clone(),
                        shortcut.clone(),
                        format!("run -b 'echo {} | base64 -d | sh'", encoded_command),
                    ]
                }
                MenuType::NoDim { name } => {
                    vec![format!("-#[nodim]{}", name), "".to_string(), "".to_string()]
                }
                MenuType::Seperate {} => vec!["".to_string()],
            })
            .flatten()
            .collect()
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
        arguments.push("".to_string()); // We have to add seperator here before menu items
        arguments.append(&mut Self::construct_menu_arguments(
            &menu.items,
            &menu.conf_path,
            &menu.cwd,
            &menu.cli_x,
            &menu.cli_y,
        ));
        if verbose > &0 {
            println!("Running: {}", arguments.join(" "));
        }

        self._run(arguments, false)
    }
}
