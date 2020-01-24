use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use dirs::{data_local_dir, home_dir};
use serde::{Deserialize, Serialize};

use crate::Result;
use template::{ProblemContext, ProblemTempl, Shell, TemplArray};

mod template;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Config {
    shell: Shell,
    session: SessionConfig,
    services: ServicesConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: load from file
        Ok(Config::default())
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let yaml_str = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", yaml_str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct SessionConfig {
    #[serde(with = "humantime_serde")]
    timeout: Duration,
    retry_limit: usize,
    #[serde(with = "humantime_serde")]
    retry_interval: Duration,
    cookies_path: PathBuf,
}

impl SessionConfig {
    fn default_cookies_path() -> PathBuf {
        if let (Some(home), Some(local)) = (home_dir(), data_local_dir()) {
            local
                .strip_prefix(&home)
                .ok()
                .and_then(|path| path.to_str())
                .map(|path| Path::new("~").join(path).join(env!("CARGO_PKG_NAME")))
        } else {
            None
        }
        .unwrap_or_else(|| {
            Path::new("~")
                .join(".local")
                .join("share")
                .join(env!("CARGO_PKG_NAME"))
        })
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        let cookies_path = Self::default_cookies_path();
        Self {
            timeout: Duration::from_secs(30),
            retry_limit: 4,
            retry_interval: Duration::from_secs(2),
            cookies_path,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct ServicesConfig {
    atcoder: AtcoderConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct AtcoderConfig {
    language: String,
    working_directory: ProblemTempl,
    src: ProblemTempl,
    compile: TemplArray<ProblemTempl, ProblemContext>,
    run: TemplArray<ProblemTempl, ProblemContext>,
}

impl Default for AtcoderConfig {
    fn default() -> Self {
        Self {
            language: "C++14 (GCC 5.4.1)".into(),
            working_directory: "{{ service.id }}/{{ contest.id | kebab_case }}/{{ problem.id | kebab_case }}".into(),
            src: "{{ service.id }}/{{ contest.id | kebab_case }}/{{ problem.id | kebab_case }}/Main.cpp".into(),
            compile: (&["g++", "-std=gnu++1y", "-O2", "-I/opt/boost/gcc/include", "-L/opt/boost/gcc/lib", "-o", "./a.out", "./Main.cpp"]).into(),
            run: (&["./a.out"]).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_default() -> anyhow::Result<()> {
        serde_yaml::to_string(&Config::default())?;
        Ok(())
    }

    #[test]
    fn exec_default_atcoder_compile() -> anyhow::Result<()> {
        let shell = Shell::default();
        let compile = AtcoderConfig::default().compile;
        let context = ProblemContext::default();
        let output = shell.exec_templ_arr(&compile, &context)?;
        println!("{:?}", output);
        // TODO: assert success
        Ok(())
    }
}
