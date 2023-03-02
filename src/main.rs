mod shell;

use std::io::Write;

mod pipe;
mod show;
mod tmux;

use anyhow::Result;
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
                .arg(arg!(-v --verbose ... "Verbose mode"))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("popup")
                .about("Show popup")
                .arg(arg!(--cmd <CMD> "Command to run"))
                .arg(arg!(--x <X> "X position"))
                .arg(arg!(--y <Y> "Y position"))
                .arg(arg!(--w <W> "Width"))
                .arg(arg!(--h <H> "Height"))
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
            Command::new("input")
                .about("Run a command")
                .arg(arg!(--key <KEY> "Key to show").required(true).num_args(..))
                .arg_required_else_help(true),
        )
}

fn get_inputs(value_refs: Option<ValuesRef<String>>) -> Vec<String> {
    let mut inputs = vec![];
    if let Some(values) = value_refs {
        for value in values {
            inputs.push(value.to_string());
        }
    }
    inputs
}

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("show", sub_matches)) => {
            let working_dir = canonicalize(PathBuf::from(
                sub_matches.get_one::<String>("working_dir").unwrap(),
            ))?;
            let path = PathBuf::from(sub_matches.get_one::<String>("menu").unwrap());
            let verbose = sub_matches.get_one::<u8>("verbose").unwrap();

            let menus = Menus::load(path, working_dir).expect("Failed to load menus");
            let tmux = Tmux::new();

            tmux.display_menu(&menus, verbose)?;
        }
        Some(("popup", sub_matches)) => {
            let tmux = Tmux::new();

            let working_dir = canonicalize(PathBuf::from(
                sub_matches.get_one::<String>("working_dir").unwrap(),
            ))?;

            // create pipe
            pipe::create()?;

            let mut base_arguments = vec!["input".to_string(), "--key".to_string()];
            base_arguments.extend(get_inputs(sub_matches.get_many::<String>("key")));
            let cmd_to_run_input_of_this = run_this_with(&working_dir, base_arguments)?;

            // Trying to receive the input
            let (tx, rx) = channel::<()>();
            let reader = thread::spawn(move || pipe::read(rx).expect("Failed to read pipe"));

            tmux.display_popup(cmd_to_run_input_of_this, &Position::wh(50, 3), true)
                .expect("Failed to run command");

            // Send the signal to stop reading
            let _ = tx.send(());

            let result = reader.join().expect("Failed to join reader thread");

            if result == "" {
                return Ok(());
            }

            let mut cmd = sub_matches
                .get_one::<String>("cmd")
                .expect("CMD is required")
                .to_string();

            let received_inputs: HashMap<String, String> = serde_yaml::from_str(&result)?;
            for (key, value) in received_inputs {
                cmd = cmd.replace(&format!("%%{}%%", key), &value);
            }
            cmd = format!(
                "cd {} && {}",
                working_dir.to_str().unwrap().to_string(),
                cmd
            );

            let x = sub_matches.get_one::<String>("x").unwrap().clone();
            let y = sub_matches.get_one::<String>("y").unwrap().clone();
            let w = Some(sub_matches.get_one::<String>("w").unwrap().clone());
            let h = Some(sub_matches.get_one::<String>("h").unwrap().clone());
            let e = *sub_matches.get_one::<u8>("exit").unwrap() == 1;

            pipe::remove()?;

            tmux.display_popup(cmd, &Position { x, y, w, h }, e)
                .expect("Failed to display popup");
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

                // Clear the line
                print!("\x1B[2J");

                // remove new line
                received_inputs.insert(key, input.trim().to_string());
            }

            let serialized_result = to_string(&received_inputs)?;
            pipe::write(serialized_result)?
        }
        _ => unreachable!(),
    }

    Ok(())
}
