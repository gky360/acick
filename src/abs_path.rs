use std::env::current_dir;
use std::fmt;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context as _};
use serde::Serialize;

use crate::{Console, Result};

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
    pub fn try_new(path: PathBuf) -> Result<Self> {
        // TODO: use shellexpand, follow symlinks
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(anyhow!("Path is not absolute : {}", path.display()))
        }
    }

    pub fn cwd() -> Result<Self> {
        let dir = current_dir()?;
        Self::try_new(dir)
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        // TODO: use shellexpand, follow symlinks
        Self(self.0.join(path))
    }

    pub fn save_pretty(
        &self,
        base_dir: &AbsPathBuf,
        overwrite: bool,
        save: impl FnOnce(File) -> Result<()>,
        cnsl: &mut Console,
    ) -> Result<bool> {
        write!(
            cnsl,
            "Saving {} ... ",
            self.strip_prefix(base_dir).display()
        )?;
        let is_existed = self.as_ref().is_file();
        let is_saved = if !overwrite && is_existed {
            false
        } else {
            self.create_dir_all_and_open(false, true)
                .with_context(|| format!("Could not create file : {}", self))
                .and_then(save)?;
            true
        };
        let msg = if is_saved {
            if is_existed {
                "overwritten"
            } else {
                "saved"
            }
        } else {
            "already exists"
        };
        writeln!(cnsl, "{}", msg)?;
        Ok(is_saved)
    }

    pub fn create_dir_all_and_open(&self, is_read: bool, is_write: bool) -> io::Result<File> {
        if let Some(dir) = self.0.parent() {
            create_dir_all(&dir)?;
        }
        let file = OpenOptions::new()
            .read(is_read)
            .write(is_write)
            .create(true)
            .open(&self.0)?;
        Ok(file)
    }

    pub fn strip_prefix(&self, base: &AbsPathBuf) -> &Path {
        self.0
            .strip_prefix(&base.0)
            .unwrap_or_else(|_| self.0.as_path())
    }
}

impl AsRef<PathBuf> for AbsPathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl fmt::Display for AbsPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

pub trait ToAbs {
    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf;
}

impl<T: AsRef<Path>> ToAbs for T {
    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf {
        base.join(self)
    }
}
