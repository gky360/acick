use std::env::current_dir;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
    pub fn cwd() -> Result<Self> {
        current_dir()
            .and_then(|dir| {
                if dir.is_absolute() {
                    Ok(Self(dir))
                } else {
                    Err(io::ErrorKind::Other.into())
                }
            })
            .map_err(Into::into)
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> AbsPathBuf {
        // TODO: use shellexpand, follow symlinks
        Self(self.as_ref().join(path))
    }
}

impl AsRef<PathBuf> for AbsPathBuf {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

pub trait ToAbs {
    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf;
}

impl<T: AsRef<Path>> ToAbs for T {
    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf {
        // TODO: use shellexpand, follow symlinks
        if self.as_ref().is_absolute() {
            AbsPathBuf(self.as_ref().to_owned())
        } else {
            base.join(self)
        }
    }
}
