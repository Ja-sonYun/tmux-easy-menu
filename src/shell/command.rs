use anyhow::Result;
use std::process::{Child, Command};

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

pub fn join_shell_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn run_command(command: String) -> Result<String> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn spawn_binary(command: String, args: Vec<String>, non_block: bool) -> Result<Child> {
    let mut output = Command::new(command).args(args).spawn()?;

    if !non_block {
        output.wait()?;
    }

    Ok(output)
}
