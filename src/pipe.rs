use anyhow::Result;
use nix::sys::select;
use nix::sys::stat;
use nix::sys::time::TimeVal;
use nix::sys::time::TimeValLike;
use nix::unistd::mkfifo;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

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
        .write(value.as_bytes())?;

    Ok(())
}

pub fn read(rx: Receiver<()>) -> Result<String> {
    let mut reader = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(PIPE_PATH)?;
    let mut fds = select::FdSet::new();
    fds.insert(reader.as_raw_fd());

    let mut buffer = String::new();

    loop {
        let mut timeout = TimeVal::milliseconds(50);
        select::select(None, Some(&mut fds), None, None, Some(&mut timeout))?;

        let mut buf = [0; 1024];
        let read = reader.read(&mut buf)?;

        if read > 0 {
            buffer.push_str(std::str::from_utf8(&buf[..read])?);
            break;
        }

        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    Ok(buffer)
}
