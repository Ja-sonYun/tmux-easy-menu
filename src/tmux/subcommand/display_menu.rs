use std::path::PathBuf;
use std::process::Child;

use crate::show::construct_menu::{MenuType, Menus};
use crate::tmux::Tmux;
use anyhow::Result;

static DISPLAY_MENU: &str = "display-menu";

impl Tmux {
    fn construct_menu_arguments(
        menu_items: &Vec<MenuType>,
        prev_path: &PathBuf,
        cwd: &PathBuf,
    ) -> Vec<String> {
        menu_items
            .iter()
            .map(|menu| match menu {
                MenuType::Menu { name, shortcut, .. } => vec![
                    name.clone(),
                    shortcut.clone(),
                    format!(
                        "run -b '{}'",
                        menu.get_execute_command(prev_path, cwd).unwrap()
                    ),
                ],
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

    pub fn display_menu(&self, menu: &Menus) -> Result<Child> {
        let mut arguments = vec![DISPLAY_MENU.to_string()];

        arguments.append(&mut Self::construct_title_arguments(&menu.title));
        arguments.append(&mut Self::construct_position_arguments(&menu.position));
        arguments.push("".to_string()); // We have to add seperator here before menu items
        arguments.append(&mut Self::construct_menu_arguments(
            &menu.items,
            &menu.conf_path,
            &menu.cwd,
        ));

        self._run(arguments, false)
    }
}
