use crate::shell::{shell_join, shell_quote};
use anyhow::Result;
use std::env::current_exe;
use std::path::PathBuf;

pub fn run_this_with(on_dir: &PathBuf, args: Vec<String>) -> Result<String> {
    let this_binary = current_exe()?;
    let this_binary = this_binary
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("current executable path is not valid UTF-8"))?;
    run_this_binary_with(on_dir, this_binary, args)
}

pub fn run_this_binary_with(
    on_dir: &PathBuf,
    this_binary: &str,
    args: Vec<String>,
) -> Result<String> {
    let mut command = vec![this_binary.to_string()];
    command.extend(args);
    let on_dir = on_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("target directory path is not valid UTF-8"))?;
    Ok(format!(
        "cd {} && {}",
        shell_quote(on_dir),
        shell_join(&command)?
    ))
}
