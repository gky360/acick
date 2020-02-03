use std::env::current_dir;
use std::fmt;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{self, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context as _};
use serde::Serialize;

use crate::{Console, Result};

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
    pub fn try_new(path: PathBuf) -> Result<Self> {
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(anyhow!("Path is not absolute : {}", path.display()))
        }
    }

    fn expand<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        Ok(shellexpand::full(&path.as_ref().to_string_lossy())?.parse()?)
    }

    pub fn cwd() -> Result<Self> {
        Ok(Self(current_dir()?))
    }

    pub fn join_expand<P: AsRef<Path>>(&self, path: P) -> Result<Self> {
        Ok(self.join(Self::expand(path)?))
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        Self(self.0.join(path))
    }

    pub fn search_dir_contains(&self, file_name: &str) -> Option<Self> {
        for dir in self.0.ancestors() {
            let mut file_path = dir.join(file_name);
            if file_path.is_file() {
                file_path.pop();
                return Some(Self(file_path));
            }
        }
        None
    }

    pub fn save_pretty(
        &self,
        save: impl FnOnce(File) -> Result<()>,
        overwrite: bool,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut Console,
    ) -> Result<bool> {
        write!(
            cnsl,
            "Saving {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let is_existed = self.as_ref().is_file();
        let result = if !overwrite && is_existed {
            Ok(false)
        } else {
            self.create_dir_all_and_open(false, true)
                .with_context(|| format!("Could not open file : {}", self))
                .and_then(|mut file| {
                    // truncate file before write
                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    Ok(file)
                })
                .and_then(save)
                .map(|_| true)
        };
        let msg = if let Ok(is_saved) = result {
            if is_saved {
                if is_existed {
                    "overwritten"
                } else {
                    "saved"
                }
            } else {
                "already exists"
            }
        } else {
            "failed"
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    pub fn load_pretty<T>(
        &self,
        load: impl FnOnce(File) -> Result<T>,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut Console,
    ) -> Result<T> {
        write!(
            cnsl,
            "Loading {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = OpenOptions::new()
            .read(true)
            .open(&self.0)
            .with_context(|| format!("Could not open file : {}", self))
            .and_then(load);
        let msg = match result {
            Ok(_) => "loaded",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
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

    fn strip_prefix_if(&self, base: Option<&AbsPathBuf>) -> &Path {
        if let Some(base) = base {
            self.strip_prefix(base)
        } else {
            self.0.as_path()
        }
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
    fn to_abs_expand(&self, base: &AbsPathBuf) -> Result<AbsPathBuf>;

    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf;
}

impl<T: AsRef<Path>> ToAbs for T {
    fn to_abs_expand(&self, base: &AbsPathBuf) -> Result<AbsPathBuf> {
        base.join_expand(self)
    }

    fn to_abs(&self, base: &AbsPathBuf) -> AbsPathBuf {
        base.join(self)
    }
}
