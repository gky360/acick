use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher as _};
use std::process::{Command, Output};
use std::sync::Mutex;

use anyhow::Context as _;
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

pub trait Expand<C: Serialize> {
    fn get_template(&self) -> &str;

    fn expand(&self, context: &C) -> Result<String> {
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
            serde_json::to_string(context).expect("Failed to serialize context")
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdContext {
    command: String,
}

impl CmdContext {
    pub fn new<T: ToString>(command: T) -> Self {
        Self {
            command: command.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdTempl(String);

impl Expand<CmdContext> for CmdTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

impl From<String> for CmdTempl {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Service {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contest {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Problem {
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemContext {
    service: Service,
    contest: Contest,
    problem: Problem,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemTempl(String);

impl Expand<ProblemContext> for ProblemTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

impl From<String> for ProblemTempl {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellArray(Vec<String>);

impl ShellArray {
    pub fn exec(&self) -> Result<Output> {
        let output = Command::new(&self.0[0])
            .args(&self.0[1..])
            .output()
            .context(format!("Failed to execute command: \"{}\"", self))?;
        Ok(output)
    }
}

impl From<ShellArray> for String {
    fn from(shell_array: ShellArray) -> String {
        format!("{}", shell_array)
    }
}

impl fmt::Display for ShellArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .enumerate()
            .try_for_each(|(i, c)| write!(f, "{}{}", if i == 0 { "" } else { " " }, c))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShellTemplArray<T>(Vec<T>);

impl<T: Expand<CmdContext>> ShellTemplArray<T> {
    pub fn expand_all(&self, context: &CmdContext) -> Result<ShellArray> {
        let array = self
            .0
            .iter()
            .map(|c| c.expand(context))
            .collect::<Result<Vec<String>>>()
            .context("Failed to expand command template")?;
        Ok(ShellArray(array))
    }
}

impl<I, S, T: From<String>> From<I> for ShellTemplArray<T>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        ShellTemplArray(
            value
                .into_iter()
                .map(|s| s.as_ref().to_string().into())
                .collect(),
        )
    }
}

impl<T: fmt::Display> From<ShellTemplArray<T>> for String {
    fn from(shell: ShellTemplArray<T>) -> String {
        format!("{}", shell)
    }
}

impl<T: fmt::Display> fmt::Display for ShellTemplArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .enumerate()
            .try_for_each(|(i, c)| write!(f, "{}{}", if i == 0 { "" } else { " " }, c))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Config {
    shell: ShellTemplArray<CmdTempl>,
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
    compile: ShellTemplArray<ProblemTempl>,
    run: ShellTemplArray<ProblemTempl>,
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
    fn expand_cmd_template() -> anyhow::Result<()> {
        let shell: ShellTemplArray<CmdTempl> = (&["/bin/bash", "-c", "{{ command }}"]).into();
        let cmd_context = CmdContext::new("echo hello");
        shell.expand_all(&cmd_context)?;
        Ok(())
    }

    #[test]
    fn expand_cmd_template_failure() -> anyhow::Result<()> {
        let shell: ShellTemplArray<CmdTempl> =
            (&["/bin/bash", "-c", "{{ some_undefined_variable }}"]).into();
        let cmd_context = CmdContext::new("echo hello");
        assert!(shell.expand_all(&cmd_context).is_err());
        Ok(())
    }

    #[test]
    fn exec_command() -> anyhow::Result<()> {
        let shell = ShellArray(
            (&["/bin/bash", "-c", "echo hello"])
                .iter()
                .map(|c| (*c).to_string())
                .collect(),
        );
        let output = shell.exec()?;
        println!("{:?}", output);
        assert!(output.status.success());
        Ok(())
    }
}
