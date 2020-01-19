use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Mutex;
use std::{env, fmt};

use anyhow::Context as _;
use heck::{CamelCase as _, KebabCase as _, MixedCase as _, SnakeCase as _};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::model::{Contest, Problem, Service};
use crate::Result;

macro_rules! register_case_conversion {
    ($renderer:ident, $case_name:expr, $func:ident) => {
        let filter_name = format!("{}_case", $case_name);
        $renderer.register_filter(
            &filter_name,
            |value: &tera::Value, _: &HashMap<String, tera::Value>| {
                let s =
                    tera::try_get_value!(format!("{}_case", $case_name), "value", String, value);
                tera::to_value(s.$func()).map_err(|e| {
                    tera::Error::chain(
                        format!("Could not convert \"{}\" to {} case", s, $case_name),
                        e,
                    )
                })
            },
        )
    };
}

lazy_static! {
    static ref RENDERER: Mutex<Tera> = {
        let mut renderer = Tera::default();
        register_case_conversion!(renderer, "pascal", to_camel_case);
        register_case_conversion!(renderer, "kebab", to_kebab_case);
        register_case_conversion!(renderer, "camel", to_mixed_case);
        register_case_conversion!(renderer, "snake", to_snake_case);

        Mutex::new(renderer)
    };
}

pub trait Expand<C: Serialize> {
    fn get_template(&self) -> &str;

    fn expand(&self, context: &C) -> Result<String> {
        let template = self.get_template();
        let template_name = template;

        let ctx = Context::from_serialize(context).context("Could not create template context")?;

        let mut renderer = RENDERER.lock().unwrap();
        if let Err(err) = renderer.get_template(template_name) {
            if let tera::ErrorKind::TemplateNotFound(_) = err.kind {
                // need to register template because this is the first time to use it
                renderer
                    .add_raw_template(template_name, template)
                    .context("Could not build template inheritance chain")?;
            } else {
                return Err(err).context("Could not expand template")?;
            }
        };
        renderer.render(template_name, &ctx).context(format!(
            "Could not expand template with context\n    template: {}\n    context: {}",
            template,
            serde_json::to_string(context).expect("Failed to serialize context")
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Templ(String);

impl<T: ToString> From<T> for Templ {
    fn from(value: T) -> Self {
        Self(value.to_string())
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

type CmdTempl = Templ;

impl Expand<CmdContext> for CmdTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemContext {
    service: Service,
    contest: Contest,
    problem: Problem,
}

type ProblemTempl = Templ;

impl Expand<ProblemContext> for ProblemTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplArray<T>(Vec<T>);

impl<T: Expand<CmdContext>> TemplArray<T> {
    pub fn expand_all(&self, context: &CmdContext) -> Result<Vec<String>> {
        self.0.iter().map(|c| c.expand(context)).collect()
    }
}

impl<I, S, T: From<String>> From<I> for TemplArray<T>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        TemplArray(
            value
                .into_iter()
                .map(|s| s.as_ref().to_string().into())
                .collect(),
        )
    }
}

impl<T: fmt::Display> From<TemplArray<T>> for String {
    fn from(shell: TemplArray<T>) -> String {
        format!("{}", shell)
    }
}

impl<T: fmt::Display> fmt::Display for TemplArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .enumerate()
            .try_for_each(|(i, c)| write!(f, "{}{}", if i == 0 { "" } else { " " }, c))
    }
}

pub type Shell = TemplArray<CmdTempl>;

impl Shell {
    pub fn exec<T: ToString>(&self, command: T) -> Result<Output> {
        let cmd_context = CmdContext::new(command);
        let command = self
            .expand_all(&cmd_context)
            .context("Could not expand shell template")?;
        let output = Command::new(&command[0])
            .args(&command[1..])
            .output()
            .context(format!(
                "Failed to execute command: \"{}\"",
                command.join(" ")
            ))?;
        Ok(output)
    }
}

impl Default for Shell {
    fn default() -> Self {
        let env_path = env::var_os("PATH").unwrap_or_default();

        let bash = env::split_paths(&env_path)
            .chain(if cfg!(windows) {
                vec![
                    PathBuf::from(r"C:\tools\msys64\usr\bin"),
                    PathBuf::from(r"C:\msys64\usr\bin"),
                    PathBuf::from(r"C:\Program Files\Git\usr\bin"),
                ]
            } else {
                vec![]
            })
            .map(|p| {
                if cfg!(windows) {
                    p.join("bash").with_extension("exe")
                } else {
                    p.join("bash")
                }
            })
            .find(|p| p.exists() && p.to_str().is_some())
            .unwrap_or_else(|| PathBuf::from("bash"));

        (&[bash.to_str().unwrap(), "-c", "{{ command }}"]).into()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct Config {
    shell: Shell,
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
            shell: Shell::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_cmd_templ() -> anyhow::Result<()> {
        let templ = CmdTempl::from("some/{{ command }}.out");
        let cmd_context = CmdContext::new("echo hello");
        templ.expand(&cmd_context)?;
        Ok(())
    }

    #[test]
    fn expand_problem_templ() -> anyhow::Result<()> {
        use crate::model::ServiceKind;

        let templ = ProblemTempl::from("{{ service.id | snake_case }}/{{ contest.id | kebab_case }}/{{ problem.id | camel_case }}/Main.cpp");
        let service = Service::new(ServiceKind::Atcoder);
        let contest = Contest::new("arc100");
        let problem = Problem::new("a");
        let problem_context = ProblemContext {
            service,
            contest,
            problem,
        };
        templ.expand(&problem_context)?;
        Ok(())
    }

    #[test]
    fn expand_default_shell() -> anyhow::Result<()> {
        let shell = Shell::default();
        let cmd_context = CmdContext::new("echo hello");
        shell.expand_all(&cmd_context)?;
        Ok(())
    }

    #[test]
    fn expand_shell_failure() -> anyhow::Result<()> {
        let shell = Shell::from(&["/bin/bash", "-c", "{{ some_undefined_variable }}"]);
        let cmd_context = CmdContext::new("echo hello");
        assert!(shell.expand_all(&cmd_context).is_err());
        Ok(())
    }

    #[test]
    fn exec_default_shell() -> anyhow::Result<()> {
        let shell = Shell::default();
        let output = shell.exec("echo hello")?;
        println!("{:?}", output);
        assert!(output.status.success());
        Ok(())
    }
}
