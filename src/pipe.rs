use anyhow::Result;
use nix::sys::stat;
use nix::unistd::mkfifo;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;

static PIPE_PATH: &str = "/tmp/tmux-menu.pipe";

pub fn create() -> Result<()> {
    remove()?;
    mkfifo(PIPE_PATH, stat::Mode::S_IRWXU)?;

    Ok(())
}

pub fn remove() -> Result<()> {
    let path = std::path::Path::new(PIPE_PATH);
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

pub fn write(value: String) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(PIPE_PATH)?
        .write_all(value.as_bytes())?;

    Ok(())
}

pub fn read() -> Result<String> {
    let mut reader = OpenOptions::new().read(true).open(PIPE_PATH)?;

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    Ok(buffer)
}
