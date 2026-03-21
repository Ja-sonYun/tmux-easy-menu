use crate::shell::{join_shell_args, shell_quote};
use anyhow::Result;
use std::env::current_exe;
use std::path::PathBuf;

fn parse_arguments(args: Vec<String>) -> Result<String> {
    Ok(join_shell_args(&args))
}

pub fn run_this_with(_on_dir: &PathBuf, args: Vec<String>) -> Result<String> {
    let this_binary = current_exe()?;
    let args = parse_arguments(args)?;

    if args.is_empty() {
        Ok(shell_quote(this_binary.to_str().unwrap()))
    } else {
        Ok(format!(
            "{} {}",
            shell_quote(this_binary.to_str().unwrap()),
            args
        ))
    }
}
