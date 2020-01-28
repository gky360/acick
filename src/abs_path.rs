use std::env::current_dir;
use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use serde::Serialize;

use crate::Result;

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
