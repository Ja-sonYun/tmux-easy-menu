mod shell;

use std::io::Write;

mod pipe;
mod show;
mod tmux;

use anyhow::Result;
use shell::{exec_shell, run_command, shell_quote};
use show::{construct_menu::Menus, construct_position::Position, this::run_this_with};

use clap::{arg, parser::ValuesRef, Command};
use std::sync::mpsc::channel;
use tmux::Tmux;

use serde_yaml::to_string;
use std::collections::HashMap;
use std::fs::canonicalize;
use std::io;
use std::path::PathBuf;
use std::thread;

fn cli() -> Command {
    Command::new("tmux-menu")
        .about("A tmux menu")
        .subcommand_required(true)
        .subcommand(
            Command::new("show")
                .about("Show the menu")
                .arg(arg!(--menu <MENU> "Path to the menu file").required(true))
                .arg(
                    arg!(--working_dir <DIR> "Working directory")
                        .required(false)
                        .default_value("."),
                )
                .arg(arg!(-x --x <X> "X position for display-menu").required(false))
                .arg(arg!(-y --y <Y> "Y position for display-menu").required(false))
                .arg(arg!(-v --verbose ... "Verbose mode"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("popup")
                .about("Show popup")
                .arg(arg!(--cmd <CMD> "Command to run"))
                .arg(arg!(-x --x <X> "X position"))
                .arg(arg!(-y --y <Y> "Y position"))
                .arg(arg!(--w <W> "Width"))
                .arg(arg!(--h <H> "Height"))
                .arg(arg!(--border <BORDER> "Border"))
                .arg(arg!(--session_name <SESSION> "Popup session name").required(false))
                .arg(arg!(--key <KEY> "Key to show").num_args(..))
                .arg(arg!(-E --exit ... "Exit after command"))
                .arg(
                    arg!(--working_dir <DIR> "Working directory")
                        .required(false)
                        .default_value("."),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("select")
                .about("Run a selected menu item")
                .arg(arg!(--menu <MENU> "Path to the menu file").required(true))
                .arg(arg!(--index <INDEX> "Menu item index").required(true))
                .arg(
                    arg!(--working_dir <DIR> "Working directory")
                        .required(false)
                        .default_value("."),
                )
                .arg(arg!(-x --x <X> "X position for display-menu").required(false))
                .arg(arg!(-y --y <Y> "Y position for display-menu").required(false))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("input")
                .about("Run a command")
                .arg(arg!(--key <KEY> "Key to show").required(true).num_args(..))
                .arg_required_else_help(true),
        )
}

fn get_inputs(value_refs: Option<ValuesRef<String>>) -> Vec<String> {
    value_refs
        .map(|values| values.map(ToString::to_string).collect())
        .unwrap_or_default()
}

fn apply_cli_position(menus: &mut Menus, x: Option<String>, y: Option<String>) {
    if let Some(x) = x {
        menus.position.x = x.clone();
        menus.cli_x = Some(x);
    }
    if let Some(y) = y {
        menus.position.y = y.clone();
        menus.cli_y = Some(y);
    }
}

fn position_from_geometry(geometry: &str) -> Option<Position> {
    let mut values = geometry.split_whitespace();
    let position = Position {
        x: values.next()?.to_string(),
        y: values.next()?.to_string(),
        w: Some(values.next()?.to_string()),
        h: Some(values.next()?.to_string()),
    };
    values.next().is_none().then_some(position)
}

fn saved_popup_position(session_name: Option<&String>) -> Option<Position> {
    let key: String = session_name?
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let geometry = run_command(format!("tmux show-options -gqv @popup_geom_{key}")).ok()?;
    position_from_geometry(&geometry)
}

fn run_show(
    menu: String,
    working_dir: String,
    x: Option<String>,
    y: Option<String>,
    verbose: u8,
) -> Result<()> {
    let working_dir = canonicalize(PathBuf::from(working_dir))?;
    let path = canonicalize(PathBuf::from(menu))?;
    let mut menus = Menus::load(path, working_dir)?;

    apply_cli_position(&mut menus, x, y);

    let tmux = Tmux::new();
    tmux.display_menu(&menus, &verbose)?;
    Ok(())
}

fn run_select(
    menu: String,
    working_dir: String,
    index: usize,
    x: Option<String>,
    y: Option<String>,
) -> Result<()> {
    let working_dir = canonicalize(PathBuf::from(working_dir))?;
    let path = canonicalize(PathBuf::from(menu))?;
    let mut menus = Menus::load_for_select(path, working_dir)?;

    apply_cli_position(&mut menus, x, y);

    let menu = menus
        .items
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("Menu item index out of range: {index}"))?;
    let command = menu.get_execute_command(
        &menus.conf_path,
        &menus.cwd,
        menus.cli_x.as_deref(),
        menus.cli_y.as_deref(),
    )?;

    exec_shell(command)?;
    Ok(())
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("show", sub_matches)) => {
            run_show(
                sub_matches.get_one::<String>("menu").unwrap().clone(),
                sub_matches
                    .get_one::<String>("working_dir")
                    .unwrap()
                    .clone(),
                sub_matches.get_one::<String>("x").cloned(),
                sub_matches.get_one::<String>("y").cloned(),
                *sub_matches.get_one::<u8>("verbose").unwrap(),
            )?;
        }
        Some(("popup", sub_matches)) => {
            let tmux = Tmux::new();

            let working_dir = canonicalize(PathBuf::from(
                sub_matches.get_one::<String>("working_dir").unwrap(),
            ))?;

            let border = sub_matches.get_one::<String>("border").unwrap().clone();
            let keys = get_inputs(sub_matches.get_many::<String>("key"));

            let mut cmd = sub_matches
                .get_one::<String>("cmd")
                .expect("CMD is required")
                .to_string();

            let x = sub_matches.get_one::<String>("x").unwrap().clone();
            let y = sub_matches.get_one::<String>("y").unwrap().clone();
            let w = Some(sub_matches.get_one::<String>("w").unwrap().clone());
            let h = Some(sub_matches.get_one::<String>("h").unwrap().clone());
            let e = *sub_matches.get_one::<u8>("exit").unwrap() == 1;

            let position = saved_popup_position(sub_matches.get_one::<String>("session_name"))
                .unwrap_or(Position { x, y, w, h });

            // consumed by the tmux-side popup-move keybinding; only effective for `session: true` popups
            let raw_geom = format!(
                "{} {} {} {}",
                position.x,
                position.y,
                position.w.as_deref().unwrap_or(""),
                position.h.as_deref().unwrap_or("")
            );
            let _ = run_command(format!(
                "tmux set -g @popup_client \"$(tmux display-message -p '#{{client_name}}')\"; \
                 tmux set -g @popup_pending_geom {}; tmux set -g @popup_pending_border {}",
                shell_quote(&raw_geom),
                shell_quote(&border)
            ));

            if !keys.is_empty() {
                pipe::create()?;

                let mut base_arguments = vec!["input".to_string(), "--key".to_string()];
                base_arguments.extend(keys);
                let cmd_to_run_input_of_this = run_this_with(&working_dir, base_arguments)?;

                let (tx, rx) = channel::<()>();
                let reader = thread::spawn(move || pipe::read(rx).expect("Failed to read pipe"));

                tmux.display_popup(cmd_to_run_input_of_this, &position, &border, true)
                    .expect("Failed to run command");

                let _ = tx.send(());

                let result = reader.join().expect("Failed to join reader thread");

                if result.is_empty() {
                    pipe::remove()?;
                    return Ok(());
                }

                let received_inputs: HashMap<String, String> = serde_yaml::from_str(&result)?;
                for (key, value) in received_inputs {
                    cmd = cmd.replace(&format!("%%{}%%", key), &value);
                }
            }
            let working_dir = working_dir
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("working directory path is not valid UTF-8"))?;
            cmd = format!("cd {} && {}", shell_quote(working_dir), cmd);

            pipe::remove()?;

            tmux.display_popup(cmd, &position, &border, e)
                .expect("Failed to display popup");
        }
        Some(("select", sub_matches)) => {
            run_select(
                sub_matches.get_one::<String>("menu").unwrap().clone(),
                sub_matches
                    .get_one::<String>("working_dir")
                    .unwrap()
                    .clone(),
                sub_matches
                    .get_one::<String>("index")
                    .unwrap()
                    .parse::<usize>()?,
                sub_matches.get_one::<String>("x").cloned(),
                sub_matches.get_one::<String>("y").cloned(),
            )?;
        }
        Some(("input", sub_matches)) => {
            let mut received_inputs: HashMap<String, String> = HashMap::new();

            for key in get_inputs(sub_matches.get_many::<String>("key")) {
                print!(" > {}: ", key);
                std::io::stdout().flush().unwrap();

                let mut input = String::new();

                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                print!("\x1B[2J");

                received_inputs.insert(key, input.trim().to_string());
            }

            let serialized_result = to_string(&received_inputs)?;
            pipe::write(serialized_result)?
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::position_from_geometry;

    #[test]
    fn popup_geometry_requires_four_values() {
        let position = position_from_geometry("10 20 80 24").unwrap();

        assert_eq!(position.x, "10");
        assert_eq!(position.y, "20");
        assert_eq!(position.w.as_deref(), Some("80"));
        assert_eq!(position.h.as_deref(), Some("24"));
        assert!(position_from_geometry("10 20 80").is_none());
        assert!(position_from_geometry("10 20 80 24 extra").is_none());
    }
}
