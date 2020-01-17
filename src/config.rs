use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher as _};
use std::process::{Command, Output};
use std::sync::Mutex;

use anyhow::{anyhow, Context as _};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::Result;

lazy_static! {
    static ref RENDERER: Mutex<Tera> = Mutex::new(Tera::default());
}

fn calc_hash<H: Hash>(h: H) -> String {
    let mut hasher = DefaultHasher::new();
    h.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

trait Expand<C: Serialize> {
    fn get_template(&self) -> &str;

    fn expand(&self, context: C) -> Result<String> {
        let template = self.get_template();
        let template_hash = calc_hash(template);

        let ctx = Context::from_serialize(context).context("Could not create template context")?;

        let mut renderer = RENDERER.lock().unwrap();
        if let Err(err) = renderer.get_template(&template_hash) {
            if let tera::ErrorKind::TemplateNotFound(_) = err.kind {
                // need to register template because this is the first time to use it
                renderer
                    .add_raw_template(&template_hash, template)
                    .context("Could not build template inheritance chain")?;
            } else {
                return Err(err).context("Could not expand template")?;
            }
        };
        renderer.render(&template_hash, &ctx).context(format!(
            "Could not expand template with context.\ntemplate: {}\ncontext: {}",
            template,
            serde_json::to_string(&context).expect("Failed to serialize context")
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdTempl(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdContext {
    command: String,
}

impl Expand<CmdContext> for CmdTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellCmd<T: Expand<C>, C: Serialize>(Vec<T>);

impl<T: Expand<C>, C: Serialize> ShellCmd<T, C> {
    pub fn exec(&self, context: C) -> Result<Output> {
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
