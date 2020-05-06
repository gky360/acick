use std::env::current_dir;
use std::fmt;
use std::fs;
use std::io::{self, Seek as _, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Context as _};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::{Error, Result};

/// Wraps `shellexpand::full` method.
fn expand<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    Ok(shellexpand::full(&path.as_ref().to_string_lossy())?.parse()?)
}

/// An absolute (not necessarily canonicalized) path that may or may not exist.
#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
    /// Construct an absolute path.
    ///
    /// Returns error if `path` is not absolute.
    ///
    /// If path need to be shell-expanded, use `AbsPathBuf::from_shell_path` instead.
    pub fn try_new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(anyhow!("Path is not absolute : {}", path.display()));
        }
        let mut ret = Self(PathBuf::new());
        ret.push(path);
        Ok(ret)
    }

    /// Constructs an absolute path whilte expanding leading tilde and environment variables.
    ///
    /// Returns error if expanded `path` is not absolute.
    pub fn from_shell_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::try_new(expand(path)?)
    }

    /// Returns current directory as an absolute path.
    pub fn cwd() -> Result<Self> {
        Ok(Self(current_dir()?))
    }

    /// Joins path.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        Self(self.0.join(path))
    }

    /// Joins path while expanding leading tilde and environment variables.
    pub fn join_expand<P: AsRef<Path>>(&self, path: P) -> Result<Self> {
        Ok(self.join(expand(path)?))
    }

    fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.0.push(path)
    }

    /// Returns parent path.
    pub fn parent(&self) -> Option<Self> {
        if let Some(parent) = self.0.parent() {
            Some(Self(parent.to_owned()))
        } else {
            None
        }
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
        save: impl FnOnce(fs::File) -> Result<()>,
        overwrite: bool,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut dyn Write,
    ) -> Result<Option<bool>> {
        write!(
            cnsl,
            "Saving {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = self.save(save, overwrite);
        let msg = match result {
            Ok(Some(true)) => "overwritten",
            Ok(Some(false)) => "saved",
            Ok(None) => "already exists",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    // returns Some(true): overwritten, Some(false): created, None: skipped
    pub fn save(
        &self,
        save: impl FnOnce(fs::File) -> Result<()>,
        overwrite: bool,
    ) -> Result<Option<bool>> {
        let is_existed = self.as_ref().is_file();
        if !overwrite && is_existed {
            return Ok(None);
        }
        self.create_dir_all_and_open(false, true)
            .with_context(|| format!("Could not open file : {}", self))
            .and_then(|mut file| {
                // truncate file before write
                file.seek(SeekFrom::Start(0))?;
                file.set_len(0)?;
                Ok(file)
            })
            .and_then(save)?;
        Ok(Some(is_existed))
    }

    pub fn load_pretty<T>(
        &self,
        load: impl FnOnce(fs::File) -> Result<T>,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut dyn Write,
    ) -> Result<T> {
        write!(
            cnsl,
            "Loading {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = self.load(load);
        let msg = match result {
            Ok(_) => "loaded",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    pub fn load<T>(&self, load: impl FnOnce(fs::File) -> Result<T>) -> Result<T> {
        fs::OpenOptions::new()
            .read(true)
            .open(&self.0)
            .with_context(|| format!("Could not open file : {}", self))
            .and_then(load)
    }

    pub fn remove_dir_all_pretty(
        &self,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut dyn Write,
    ) -> Result<bool> {
        write!(
            cnsl,
            "Removing {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = self.remove_dir_all();
        let msg = match result {
            Ok(true) => "removed",
            Ok(false) => "not existed",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    fn remove_dir_all(&self) -> Result<bool> {
        if !self.as_ref().exists() {
            return Ok(false);
        }
        fs::remove_dir_all(self.as_ref())?;
        Ok(true)
    }

    pub fn remove_file_pretty(
        &self,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut dyn Write,
    ) -> Result<bool> {
        write!(
            cnsl,
            "Removing {} ... ",
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = if self.as_ref().exists() {
            self.remove_file().map(|_| true)
        } else {
            Ok(false)
        };
        let msg = match result {
            Ok(true) => "removed",
            Ok(false) => "not existed",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    fn remove_file(&self) -> Result<()> {
        fs::remove_file(self.as_ref())?;
        Ok(())
    }

    pub fn move_from_pretty(
        &self,
        from: &AbsPathBuf,
        base_dir: Option<&AbsPathBuf>,
        cnsl: &mut dyn Write,
    ) -> Result<()> {
        write!(
            cnsl,
            "Moving {} to {} ... ",
            from.strip_prefix_if(base_dir).display(),
            self.strip_prefix_if(base_dir).display()
        )?;
        let result = self.move_from(from);
        let msg = match result {
            Ok(_) => "moved",
            Err(_) => "failed",
        };
        writeln!(cnsl, "{}", msg)?;
        result
    }

    fn move_from(&self, from: &AbsPathBuf) -> Result<()> {
        fs::rename(from.as_ref(), self.as_ref())?;
        Ok(())
    }

    pub fn create_dir_all_and_open(&self, is_read: bool, is_write: bool) -> io::Result<fs::File> {
        if let Some(dir) = self.parent() {
            dir.create_dir_all()?
        }
        self.open(is_read, is_write)
    }

    pub fn create_dir_all(&self) -> io::Result<()> {
        fs::create_dir_all(self.as_ref())
    }

    fn open(&self, is_read: bool, is_write: bool) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .read(is_read)
            .write(is_write)
            .create(true)
            .open(&self.0)
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

impl FromStr for AbsPathBuf {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::from_shell_path(s)
    }
}

impl<'de> Deserialize<'de> for AbsPathBuf {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl fmt::Display for AbsPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::*;

    use crate::assert_matches;

    lazy_static! {
        static ref DRIVE: String = std::env::var("ACICK_TEST_DRIVE").unwrap_or_else(|_| String::from("C"));
        static ref SHELL_PATH_SUCCESS_TESTS: Vec<(String, PathBuf)> = {
            let mut tests = vec![
                (prefix("/a/b"), PathBuf::from(prefix("/a/b"))),
                ("~/a/b".into(), dirs::home_dir().unwrap().join("a/b")),
                if cfg!(windows) {
                    ("$APPDATA/a/b".into(), PathBuf::from(std::env::var("APPDATA").unwrap()).join("a/b"))
                } else {
                    ("$HOME/a/b".into(), dirs::home_dir().unwrap().join("a/b"))
                },
                (prefix("/a//b"), PathBuf::from(prefix("/a/b"))),
                (prefix("/a/./b"), PathBuf::from(prefix("/a/b"))),
                (prefix("/a/b/"), PathBuf::from(prefix("/a/b"))),
                (prefix("/a/../b"), PathBuf::from(prefix("/a/../b"))),
            ];
            if cfg!(windows) {
                tests.extend_from_slice(&[(
                    format!("{}:\\a\\b", &*DRIVE),
                    PathBuf::from(format!("{}:\\a\\b", &*DRIVE)),
                )]);
            }
            tests
        };
        static ref SHELL_PATH_FAILURE_TESTS: Vec<&'static str> = {
            let mut tests = vec!["./a/b/", "a/b", "$ACICK_UNKNOWN_VAR"];
            if cfg!(windows) {
                tests.extend_from_slice(&[
                    "%APPDATA%", // do not expand windows style env var
                    "/a/b", // not absolute in windows
                ]);
            }
            tests
        };
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct TestData {
        abs_path: AbsPathBuf,
    }

    fn prefix(path: &str) -> String {
        if cfg!(windows) {
            format!("{}:{}", &*DRIVE, path)
        } else {
            path.to_string()
        }
    }

    #[test]
    fn test_try_new_success() -> anyhow::Result<()> {
        let tests = &[
            (prefix("/a/b"), prefix("/a/b")),
            (prefix("/a//b"), prefix("/a/b")),
            (prefix("/a/./b"), prefix("/a/b")),
            (prefix("/a/b/"), prefix("/a/b")),
            (prefix("/a/../b"), prefix("/a/../b")),
        ];
        for (actual, expected) in tests {
            let actual = AbsPathBuf::try_new(actual)?;
            let expected = PathBuf::from(expected);
            assert_eq!(actual.as_ref(), &expected);
        }
        Ok(())
    }

    #[test]
    fn test_try_new_failure() -> anyhow::Result<()> {
        let tests = &[
            "~/a/b",
            if cfg!(windows) {
                "$APPDATA/a/b"
            } else {
                "$HOME/a/b"
            },
            "./a/b/",
            "a/b",
            "$ACICK_UNKNOWN_VAR",
        ];
        for test in tests {
            assert_matches!(AbsPathBuf::try_new(test) => Err(_));
        }
        Ok(())
    }

    #[test]
    fn test_parent() -> anyhow::Result<()> {
        let tests = &[(prefix("/a/b"), Some(prefix("/a"))), (prefix("/"), None)];
        for (left, right) in tests {
            let actual = AbsPathBuf::try_new(left)?.parent();
            let expected = right
                .as_ref()
                .map(|path| AbsPathBuf::try_new(path).unwrap());
            assert_eq!(actual, expected);
        }
        Ok(())
    }

    #[test]
    fn test_from_str_success() -> anyhow::Result<()> {
        for (actual, expected) in SHELL_PATH_SUCCESS_TESTS.iter() {
            let actual: AbsPathBuf = actual.parse()?;
            assert_eq!(actual.as_ref(), expected);
        }
        Ok(())
    }

    #[test]
    fn test_from_str_failure() -> anyhow::Result<()> {
        for test in SHELL_PATH_FAILURE_TESTS.iter() {
            assert_matches!(AbsPathBuf::from_str(test) => Err(_));
        }
        Ok(())
    }

    #[cfg(not(windows))]
    #[test]
    fn test_serialize_success_unix() -> anyhow::Result<()> {
        let test_data = TestData {
            abs_path: AbsPathBuf::try_new("/a/b")?,
        };
        let actual = serde_yaml::to_string(&test_data)?;
        let expected = format!("---\nabs_path: {}", "/a/b");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn test_serialize_success_windows() -> anyhow::Result<()> {
        let tests = &[
            (
                format!(r#"{}:\a\b"#, &*DRIVE),
                format!(r#""{}:\\a\\b""#, &*DRIVE),
            ),
            (
                format!(r#"{}:/a/b"#, &*DRIVE),
                format!(r#""{}:/a/b""#, &*DRIVE),
            ),
        ];
        for (left, right) in tests {
            let test_data = TestData {
                abs_path: AbsPathBuf::try_new(left)?,
            };
            let actual = serde_yaml::to_string(&test_data)?;
            let expected = format!(
                r#"---
abs_path: {}"#,
                right
            );
            assert_eq!(actual, expected);
        }
        Ok(())
    }

    #[test]
    fn test_deserialize_success() -> anyhow::Result<()> {
        for (actual, expected) in SHELL_PATH_SUCCESS_TESTS.iter() {
            let yaml_str = format!("---\nabs_path: {}", actual);
            let test_data: TestData = serde_yaml::from_str(&yaml_str)?;
            assert_eq!(test_data.abs_path.as_ref(), expected);
        }
        Ok(())
    }

    #[test]
    fn test_deserialize_failure() -> anyhow::Result<()> {
        for test in SHELL_PATH_FAILURE_TESTS.iter() {
            let yaml_str = format!("abs_path: {}", test);
            let result = serde_yaml::from_str::<TestData>(&yaml_str);
            assert_matches!(result => Err(_));
        }
        Ok(())
    }

    #[test]
    fn test_display() -> anyhow::Result<()> {
        let actual: AbsPathBuf = "~/a".parse()?;
        let expected = PathBuf::from(format!("{}/a", dirs::home_dir().unwrap().display()));
        assert_eq!(actual.as_ref(), &expected);
        assert_eq!(format!("{}", actual), format!("{}", expected.display()));
        Ok(())
    }
}
