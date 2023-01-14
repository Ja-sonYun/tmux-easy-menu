use anyhow::Result;
use std::env::current_exe;
use std::path::PathBuf;

fn parse_arguments(args: Vec<String>) -> Result<String> {
    // Parse arguments here, if argument has spaces, wrap it in quotes
    // If argument has quotes, escape them. and result.
    let mut result = String::new();

    for arg in args {
        if arg.contains(" ") {
            result.push_str(&format!("\"{}\" ", arg));
        } else {
            result.push_str(&format!("{} ", arg));
        }
    }

    Ok(result)
}

pub fn run_this_with(on_dir: &PathBuf, args: Vec<String>) -> Result<String> {
    let this_binary = current_exe()?;

    // TODO: Unwrap quote
    Ok(format!(
        "cd {} && {} {}",
        on_dir.to_str().unwrap(),
        this_binary.to_str().unwrap(),
        parse_arguments(args)?
    ))
}
