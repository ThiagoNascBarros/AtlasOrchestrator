//! Lock exclusivo do workspace: impede abrir o mesmo `.atlasoq/` em duas instâncias.

use std::fs::{File, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::Path;

use crate::error::{Result, StoreError};

/// Mantém o lock enquanto existir.
#[derive(Debug)]
pub struct WorkspaceLock {
    _file: File,
}

impl WorkspaceLock {
    pub fn acquire(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)
            .map_err(|e| StoreError::io(path, e))?;
        // SAFETY: flock só opera sobre o descritor válido de `file`.
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            let err = std::io::Error::last_os_error();
            return if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                Err(StoreError::AlreadyOpen)
            } else {
                Err(StoreError::io(path, err))
            };
        }
        Ok(Self { _file: file })
    }
}
