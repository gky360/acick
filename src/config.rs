use serde::{Deserialize, Serialize};

macro_rules! vec_string {
    ($($str:expr),*) => ({
        vec![$(String::from($str),)*] as Vec<String>
    });
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Config {
    shell: Vec<String>,
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
            shell: vec_string!["/bin/bash", "-c", "{{ command }}"],
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
    src: String,
    compile: Vec<String>,
    run: Vec<String>,
    working_directory: String,
}

impl Default for AtcoderConfig {
    fn default() -> Self {
        Self {
            language: "C++14 (GCC 5.4.1)".to_string(),
            working_directory: "{{ service.id }}/{{ contest.id | kebab_case }}/{{ problem.id | kebab_case }}".to_string(),
            src: "{{ service.id }}/{{ contest.id | kebab_case }}/{{ problem.id | kebab_case }}/Main.cpp".to_string(),
            compile: vec_string!["g++", "-std=gnu++1y", "-O2", "-I/opt/boost/gcc/include", "-L/opt/boost/gcc/lib", "-o", "./a.out", "./Main.cpp"],
            run: vec_string!["./a.out"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_default() -> anyhow::Result<()> {
        let conf: Config = toml::from_str("")?;
        assert_eq!(conf, Config::default());
        Ok(())
    }
}
