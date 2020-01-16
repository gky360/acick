use std::fmt;
use std::process::{Command, Output};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellCmd(Vec<String>);

impl ShellCmd {
    pub fn exec(&self) -> Result<Output> {
        if self.0.is_empty() {
            return Err(anyhow!("Empty command"));
        }
        let output = Command::new(&self.0[0])
            .args(&self.0[1..])
            .output()
            .context(format!("Failed to execute command: {}", self))?;
        Ok(output)
    }
}

impl<I, S> From<I> for ShellCmd
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        ShellCmd(value.into_iter().map(|s| s.as_ref().into()).collect())
    }
}

impl From<ShellCmd> for String {
    fn from(shell: ShellCmd) -> String {
        shell.0.join(" ")
    }
}

impl fmt::Display for ShellCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Config {
    shell: ShellCmd,
    services: ServicesConfig,
}

impl Config {
    pub fn load() -> Self {
        // TODO: load from file
        Config::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: (&["/bin/bash", "-c", "{{ command }}"]).into(),
            services: ServicesConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct ServicesConfig {
    atcoder: AtcoderConfig,
}

impl Default for ServicesConfig {
    fn default() -> Self {
        Self {
            atcoder: AtcoderConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct AtcoderConfig {
    language: String,
    working_directory: String,
    src: String,
    compile: ShellCmd,
    run: ShellCmd,
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
    fn deserialize_default_succeeds() -> anyhow::Result<()> {
        let conf: Config = serde_yaml::from_str("")?;
        assert_eq!(conf, Config::default());
        Ok(())
    }
}
