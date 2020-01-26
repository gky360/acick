use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Mutex;
use std::{env, fmt};

use anyhow::Context as _;
use heck::{CamelCase as _, KebabCase as _, MixedCase as _, SnakeCase as _};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::model::{Contest, Problem, Service, ServiceKind};
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
    #[allow(dead_code)]
    pub fn new(command: impl ToString) -> Self {
        Self {
            command: command.to_string(),
        }
    }
}

pub type CmdTempl = Templ;

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

impl Default for ProblemContext {
    fn default() -> Self {
        Self {
            service: Service::new(ServiceKind::Atcoder),
            contest: Contest::new("arc100".into()),
            problem: Problem::new("c".into(), "Linear Approximation".into(), Vec::new()),
        }
    }
}

pub type ProblemTempl = Templ;

impl Expand<ProblemContext> for ProblemTempl {
    fn get_template(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TemplArray<T: Expand<C>, C: Serialize>(Vec<T>, #[serde(skip)] PhantomData<C>);

impl<T: Expand<C>, C: Serialize> TemplArray<T, C> {
    #[allow(dead_code)]
    pub fn expand_all(&self, context: &C) -> Result<Vec<String>> {
        self.0.iter().map(|c| c.expand(context)).collect()
    }
}

impl<I, S, T: Expand<C> + From<String>, C: Serialize> From<I> for TemplArray<T, C>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        let arr = value
            .into_iter()
            .map(|s| s.as_ref().to_string().into())
            .collect();
        TemplArray(arr, PhantomData)
    }
}

impl<T: Expand<C> + fmt::Display, C: Serialize> fmt::Display for TemplArray<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .enumerate()
            .try_for_each(|(i, c)| write!(f, "{}{}", if i == 0 { "" } else { " " }, c))
    }
}

pub type Shell = TemplArray<CmdTempl, CmdContext>;

impl Shell {
    #[allow(dead_code)]
    pub fn exec(&self, command: impl ToString) -> Result<Output> {
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

    #[allow(dead_code)]
    pub fn exec_templ_arr<T: Expand<C>, C: Serialize>(
        &self,
        templ_arr: &TemplArray<T, C>,
        context: &C,
    ) -> Result<Output> {
        let command = templ_arr
            .expand_all(context)
            .context("Could not expand command template")?;
        self.exec(command.join(" "))
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
        let templ = ProblemTempl::from("{{ service.id | snake_case }}/{{ contest.id | kebab_case }}/{{ problem.id | camel_case }}/Main.cpp");
        let problem_context = ProblemContext::default();
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
