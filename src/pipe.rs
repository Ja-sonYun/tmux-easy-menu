use anyhow::Result;
use nix::sys::stat;
use nix::unistd::mkfifo;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;

static PIPE_PATH: &str = "/tmp/tmux-menu.pipe";

pub fn mkpipe() -> Result<()> {
    remove_pipe()?;
    mkfifo(PIPE_PATH, stat::Mode::S_IRWXU)?;

    Ok(())
}

pub fn remove_pipe() -> Result<()> {
    let path = std::path::Path::new(PIPE_PATH);
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

pub fn write_pipe(value: String) -> Result<()> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(PIPE_PATH)?
        .write_all(value.as_bytes())?;

    Ok(())
}

pub fn read_pipe() -> Result<String> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(PIPE_PATH)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
