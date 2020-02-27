use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{env, fmt};

use anyhow::Context as _;
use heck::{CamelCase as _, KebabCase as _, MixedCase as _, SnakeCase as _};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::Tera;
use tokio::process::Command;

use crate::model::{Contest, ContestId, Problem, ProblemId, Service, ServiceKind};
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
        register_case_conversion!(renderer, "camel", to_mixed_case);
        register_case_conversion!(renderer, "pascal", to_camel_case);
        register_case_conversion!(renderer, "snake", to_snake_case);
        register_case_conversion!(renderer, "kebab", to_kebab_case);

        Mutex::new(renderer)
    };
}

pub trait Expand<'a> {
    type Context: Serialize + 'a;

    fn get_template(&self) -> &str;

    fn is_empty(&self) -> bool {
        self.get_template().is_empty()
    }

    fn expand(&self, context: &Self::Context) -> Result<String> {
        let template = self.get_template();
        let template_name = template;

        let ctx =
            tera::Context::from_serialize(context).context("Could not create template context")?;

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

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdContext<'a> {
    command: &'a str,
}

impl<'a> CmdContext<'a> {
    pub fn new(command: &'a str) -> Self {
        Self { command }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CmdTempl(String);

impl<'a> Expand<'a> for CmdTempl {
    type Context = CmdContext<'a>;

    fn get_template(&self) -> &str {
        &self.0
    }
}

impl<'a, T: Into<String>> From<T> for CmdTempl {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for CmdTempl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetContext<'a> {
    #[serde(rename = "service")]
    service_id: ServiceKind,
    #[serde(rename = "contest")]
    contest_id: &'a ContestId,
    #[serde(rename = "problem")]
    problem_id: &'a ProblemId,
}

impl<'a> TargetContext<'a> {
    pub fn new(
        service_id: ServiceKind,
        contest_id: &'a ContestId,
        problem_id: &'a ProblemId,
    ) -> Self {
        Self {
            service_id,
            contest_id,
            problem_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetTempl(String);

impl TargetTempl {
    pub fn expand_with(
        &self,
        service_id: ServiceKind,
        contest_id: &ContestId,
        problem_id: &ProblemId,
    ) -> Result<String> {
        self.expand(&TargetContext {
            service_id,
            contest_id,
            problem_id,
        })
    }
}

impl<'a> Expand<'a> for TargetTempl {
    type Context = TargetContext<'a>;

    fn get_template(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for TargetTempl {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for TargetTempl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemContext<'a> {
    service: &'a Service,
    contest: &'a Contest,
    problem: &'a Problem,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemTempl(String);

impl ProblemTempl {
    pub fn expand_with(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
    ) -> Result<String> {
        self.expand(&ProblemContext {
            service,
            contest,
            problem,
        })
    }
}

impl<'a> Expand<'a> for ProblemTempl {
    type Context = ProblemContext<'a>;

    fn get_template(&self) -> &str {
        &self.0
    }
}

impl<T: Into<String>> From<T> for ProblemTempl {
    fn from(s: T) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for ProblemTempl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TemplArray<T>(Vec<T>);

impl<'a, T: Expand<'a>> TemplArray<T> {
    pub fn expand_all(&self, context: &<T as Expand<'a>>::Context) -> Result<Vec<String>> {
        self.0.iter().map(|c| c.expand(context)).collect()
    }
}

impl<'a, I, S, T> From<I> for TemplArray<T>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    T: Expand<'a> + From<String>,
{
    fn from(value: I) -> Self {
        let arr = value
            .into_iter()
            .map(|s| s.as_ref().to_string().into())
            .collect();
        TemplArray(arr)
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
    pub fn exec(&self, cmd: &str) -> Result<Command> {
        let cmd_context = CmdContext::new(cmd);
        let cmd_expanded = self
            .expand_all(&cmd_context)
            .context("Could not expand shell template")?;
        let mut command = Command::new(&cmd_expanded[0]);
        command.args(&cmd_expanded[1..]).kill_on_drop(true);
        Ok(command)
    }

    pub fn exec_templ<'a, T: Expand<'a>>(
        &self,
        templ: &T,
        context: &<T as Expand<'a>>::Context,
    ) -> Result<Command> {
        let cmd = templ
            .expand(context)
            .context("Could not expand command template")?;
        self.exec(&cmd)
    }

    pub fn find_bash() -> PathBuf {
        let env_path = env::var_os("PATH").unwrap_or_default();
        env::split_paths(&env_path)
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
            .find(|p| p.is_file() && p.to_str().is_some())
            .unwrap_or_else(|| PathBuf::from("bash"))
    }
}

impl Default for Shell {
    fn default() -> Self {
        let bash = Self::find_bash();
        (&[bash.to_str().unwrap(), "-e", "-c", "{{ command }}"]).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DEFAULT_CONTEST, DEFAULT_PROBLEM, DEFAULT_SERVICE};

    #[test]
    fn expand_cmd_templ() -> anyhow::Result<()> {
        let templ = CmdTempl::from("some/{{ command }}.out");
        let cmd_context = CmdContext::new("echo hello");
        templ.expand(&cmd_context)?;
        Ok(())
    }

    #[test]
    fn expand_problem_templ() -> anyhow::Result<()> {
        let templ = ProblemTempl::from("{{ service.id | snake_case }}/{{ contest.id | kebab_case }}/{{ problem.id | camel_case }}/Main.cpp");
        let problem_context = ProblemContext {
            service: &DEFAULT_SERVICE,
            contest: &DEFAULT_CONTEST,
            problem: &DEFAULT_PROBLEM,
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

    #[tokio::test]
    async fn exec_default_shell() -> anyhow::Result<()> {
        let shell = Shell::default();
        let mut command = shell.exec("echo hello")?;
        let output = command.output().await?;
        println!("{:?}", output);
        assert!(output.status.success());
        Ok(())
    }
}
