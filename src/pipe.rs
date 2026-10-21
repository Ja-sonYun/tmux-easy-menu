use anyhow::Result;
use nix::sys::select;
use nix::sys::stat;
use nix::sys::time::TimeVal;
use nix::sys::time::TimeValLike;
use nix::unistd::mkfifo;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;

pub struct InputPipe {
    directory: PathBuf,
    pub path: PathBuf,
    pub reader: File,
}

impl Drop for InputPipe {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.directory);
    }
}

pub fn create() -> Result<InputPipe> {
    let mut attempt = 0_u64;
    let directory = loop {
        let path = std::env::temp_dir().join(format!("tmux-menu-{}-{attempt}", std::process::id()));
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => break path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt += 1;
            }
            Err(error) => return Err(error.into()),
        }
    };
    let path = directory.join("input");
    let reader = (|| -> Result<File> {
        mkfifo(&path, stat::Mode::S_IRUSR | stat::Mode::S_IWUSR)?;
        Ok(OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)?)
    })();
    match reader {
        Ok(reader) => Ok(InputPipe {
            directory,
            path,
            reader,
        }),
        Err(error) => {
            let _ = std::fs::remove_file(&path);
            let _ = std::fs::remove_dir(&directory);
            Err(error)
        }
    }
}

pub fn write(path: &Path, value: String) -> Result<()> {
    // Fail if the owning reader has exited, then allow writes larger than PIPE_BUF.
    let mut writer = OpenOptions::new()
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)?;
    nix::fcntl::fcntl(
        writer.as_raw_fd(),
        nix::fcntl::FcntlArg::F_SETFL(nix::fcntl::OFlag::empty()),
    )?;
    writer.write_all(value.as_bytes())?;

    Ok(())
}

pub fn read(mut reader: File, rx: Receiver<()>) -> Result<String> {
    let mut buffer = Vec::new();

    loop {
        let mut fds = select::FdSet::new();
        fds.insert(reader.as_raw_fd());
        let mut timeout = TimeVal::milliseconds(50);
        select::select(None, Some(&mut fds), None, None, Some(&mut timeout))?;

        let mut buf = [0; 4096];
        match reader.read(&mut buf) {
            Ok(0) if !buffer.is_empty() => break,
            Ok(count) => buffer.extend_from_slice(&buf[..count]),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }

        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) if buffer.is_empty() => {
                break;
            }
            _ => {}
        }
        if buffer.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    Ok(String::from_utf8(buffer)?)
}
