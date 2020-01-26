use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use dirs::{data_local_dir, home_dir};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

mod template;

use crate::abs_path::{AbsPathBuf, ToAbs as _};
use crate::service::CookieStorage;
use crate::Result;
use template::{ProblemContext, ProblemTempl, Shell, TemplArray};

#[derive(Serialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    #[get = "pub"]
    base_dir: AbsPathBuf,
    data: ConfigData,
}

impl Config {
    pub fn load(base_dir: AbsPathBuf) -> Result<Self> {
        // TODO: load from file
        Ok(Self {
            base_dir,
            data: ConfigData::default(),
        })
    }

    pub fn load_cookies(&self) -> Result<CookieStorage> {
        CookieStorage::open(&self.data.session.cookies_path.to_abs(&self.base_dir))
    }

    #[inline(always)]
    pub fn shell(&self) -> &Shell {
        &self.data.shell
    }

    #[inline(always)]
    pub fn session(&self) -> &SessionConfig {
        &self.data.session
    }

    #[inline(always)]
    pub fn services(&self) -> &ServicesConfig {
        &self.data.services
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let yaml_str = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", yaml_str)
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
struct ConfigData {
    shell: Shell,
    session: SessionConfig,
    services: ServicesConfig,
}

#[derive(Serialize, Deserialize, Getters, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct SessionConfig {
    #[get = "pub"]
    user_agent: String,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    timeout: Duration,
    #[get_copy = "pub"]
    retry_limit: usize,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    retry_interval: Duration,
    #[get = "pub"]
    cookies_path: PathBuf,
}

impl SessionConfig {
    const COOKIES_FILE_NAME: &'static str = "cookies.json";

    fn default_user_agent() -> String {
        format!(
            "{}-{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY")
        )
    }

    fn default_cookies_path() -> PathBuf {
        data_local_dir()
            .unwrap_or_else(|| {
                home_dir()
                    .expect("Could not get home dir")
                    .join(".local")
                    .join("share")
            })
            .join(env!("CARGO_PKG_NAME"))
            .join(Self::COOKIES_FILE_NAME)
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            user_agent: Self::default_user_agent(),
            timeout: Duration::from_secs(30),
            retry_limit: 4,
            retry_interval: Duration::from_secs(2),
            cookies_path: Self::default_cookies_path(),
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
        serde_yaml::to_string(&Config::load(AbsPathBuf::cwd()?)?)?;
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
