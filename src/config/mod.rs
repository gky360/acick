use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::anyhow;
use dirs::{data_local_dir, home_dir};
use getset::{CopyGetters, Getters};
use semver::Version;
use serde::{Deserialize, Serialize};

mod template;

use crate::abs_path::{AbsPathBuf, ToAbs as _};
use crate::service::CookieStorage;
use crate::Result;
use template::{ProblemTempl, Shell, TemplArray};

#[derive(Serialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
pub struct Config {
    base_dir: AbsPathBuf,
    data: ConfigData,
}

impl Config {
    pub fn load(base_dir: AbsPathBuf) -> Result<Self> {
        // TODO: load from file
        let data = ConfigData::default();
        let version_str = data.version.to_string();
        let pkg_version = env!("CARGO_PKG_VERSION");
        if version_str != pkg_version {
            Err(anyhow!(
                r#"Found mismatched version in config file.
    config version: {}
    acick version : {}
Fix version in the config file so that it matches the acick version."#,
                version_str,
                pkg_version
            ))
        } else {
            Ok(Self { base_dir, data })
        }
    }

    pub fn open_cookie_storage(&self) -> Result<CookieStorage> {
        let cookies_path = &self.data.session.cookies_path;
        CookieStorage::open(&cookies_path.to_abs(&self.base_dir))
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let yaml_str = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", yaml_str)
    }
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
#[get = "pub"]
pub struct ConfigData {
    #[serde(with = "string")]
    version: Version,
    shell: Shell,
    session: SessionConfig,
    services: ServicesConfig,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            shell: Shell::default(),
            session: SessionConfig::default(),
            services: ServicesConfig::default(),
        }
    }
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
    compile: TemplArray<ProblemTempl>,
    run: TemplArray<ProblemTempl>,
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

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template::ProblemContext;
    use crate::tests::{DEFAULT_CONTEST, DEFAULT_PROBLEM, DEFAULT_SERVICE};

    #[test]
    fn serialize_default() -> anyhow::Result<()> {
        serde_yaml::to_string(&Config::load(AbsPathBuf::cwd()?)?)?;
        Ok(())
    }

    #[test]
    fn exec_default_atcoder_compile() -> anyhow::Result<()> {
        let shell = Shell::default();
        let compile = AtcoderConfig::default().compile;
        let context = ProblemContext {
            service: &DEFAULT_SERVICE,
            contest: &DEFAULT_CONTEST,
            problem: &DEFAULT_PROBLEM,
        };
        let output = shell.exec_templ_arr(&compile, &context)?;
        println!("{:?}", output);
        // TODO: assert success
        Ok(())
    }
}
