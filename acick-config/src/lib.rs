//! Config for acick.
//!
//! ## Templates
//!
//! In some fields, you can use [Tera](https://tera.netlify.com/) template.
//! Tera template is similar to Jinja2 and Django templates.
//! See [Tera documentation](https://tera.netlify.com/docs/) for details.
//!
//! Following filters are available in addition to built-in filters of Tera.
//! - `camel` : converts string to `camelCase`
//! - `pascal` : converts string to `PascalCase`
//! - `snake` : converts string to `snake_case`
//! - `kebab` : converts string to `kebab-case`
//!
//! Available variables depend on fields.
//! See [Field features](#field-features) section for details.
//!
//! ## Field features
//!
//! Fields have following features.
//!
//! ### `[c]` Command template field
//!
//! The field is recognized as an array of Tera templates
//! with the following variables available:
//! - `command` (str): command to be executed on shell
//!
//! ### `[t]` Target template field
//!
//! The field is recognized as a Tera template
//! with the following variables available:
//! - `service` (str): id of service (e.g.: `atcoder`)
//! - `contest` (str): id of contest (e.g.: `arc100`)
//! - `problem` (str): id of problem (e.g.: `C`)
//!
//! ### `[p]` Problem template field
//!
//! The field is recognized as a Tera template
//! with the following variables available:
//! - `service` (object): object that describes service
//! - `contest` (object): object that describes contest
//! - `problem` (object): object that describes problem
//!
//! ### `[s]` Shell-expanded field
//!
//! The field is processed with shell-like expansions.
//! - Tilde `~` is expanded to the home directory.
//! - Environment variables are expanded into their values.
//!
//! When combined with Tera template,
//! the field is first processed as a template and then expanded.

use std::fmt;
use std::io::{Read as _, Write};

use anyhow::{anyhow, Context as _};
use lazy_static::lazy_static;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use acick_util::{abs_path, console, model, DATA_LOCAL_DIR};

mod session_config;
mod template;

use crate::abs_path::AbsPathBuf;
use crate::console::Console;
use crate::model::{Contest, ContestId, LangName, Problem, ProblemId, Service, ServiceKind};
pub use session_config::SessionConfig;
use template::{Expand, ProblemTempl, Shell, TargetContext, TargetTempl};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

lazy_static! {
    static ref VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub service_id: ServiceKind,
    pub contest_id: ContestId,
    pub base_dir: AbsPathBuf,
    body: ConfigBody,
}

impl Config {
    pub fn load(
        service_id: ServiceKind,
        contest_id: ContestId,
        base_dir: Option<AbsPathBuf>,
        cnsl: &mut Console,
    ) -> Result<Self> {
        let base_dir = match base_dir {
            Some(base_dir) => base_dir,
            None => ConfigBody::search(cnsl)?,
        };
        let body = ConfigBody::load(&base_dir, cnsl)?;
        Ok(Self {
            service_id,
            contest_id,
            base_dir,
            body,
        })
    }

    pub fn session(&self) -> &SessionConfig {
        &self.body.session
    }

    pub fn service(&self) -> &ServiceConfig {
        self.body.services.get(self.service_id)
    }

    pub fn move_testcases_dir(
        &self,
        problem: &Problem,
        from: &AbsPathBuf,
        cnsl: &mut Console,
    ) -> Result<bool> {
        let testcases_abs_dir = self.testcases_abs_dir(problem.id())?;
        if testcases_abs_dir.as_ref().exists() {
            let message = format!(
                "remove existing testcases dir {}?",
                testcases_abs_dir.strip_prefix(&self.base_dir).display()
            );
            if !cnsl.confirm(&message, false)? {
                return Ok(false);
            }
            testcases_abs_dir.remove_dir_all_pretty(Some(&self.base_dir), cnsl)?;
        } else if let Some(parent) = testcases_abs_dir.parent() {
            parent.create_dir_all()?;
        }

        testcases_abs_dir.move_from_pretty(from, Some(&self.base_dir), cnsl)?;

        Ok(true)
    }

    pub fn save_problem(
        &self,
        problem: &Problem,
        overwrite: bool,
        cnsl: &mut Console,
    ) -> Result<Option<bool>> {
        let problem_abs_path = self.problem_abs_path(problem.id())?;
        problem_abs_path.save_pretty(
            |file| serde_yaml::to_writer(file, &problem).context("Could not save problem as yaml"),
            overwrite,
            Some(&self.base_dir),
            cnsl,
        )
    }

    pub fn load_problem(&self, problem_id: &ProblemId, cnsl: &mut Console) -> Result<Problem> {
        let problem_abs_path = self.problem_abs_path(problem_id)?;
        let problem: Problem = problem_abs_path
            .load_pretty(
                |file| serde_yaml::from_reader(file).context("Could not read problem as yaml"),
                Some(&self.base_dir),
                cnsl,
            )
            .context(
                "Could not load problem file. \
                 Fetch problem data first by `acick fetch` command.",
            )?;
        if problem.id() != problem_id {
            Err(anyhow!(
                "Found mismatching problem id in problem file : {}",
                problem.id()
            ))
        } else {
            Ok(problem)
        }
    }

    pub fn expand_and_save_source(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
        overwrite: bool,
        cnsl: &mut Console,
    ) -> Result<Option<bool>> {
        if service.id() != self.service_id || contest.id() != &self.contest_id {
            return Err(anyhow!("Found mismatching service id or contest id"));
        }
        let source_abs_path = self.source_abs_path(problem.id())?;
        let template = match &self.service().template {
            Some(template) => template,
            None => return Ok(None), // skip if template is empty
        };
        let template_expanded = template.expand_with(service, contest, problem)?;
        source_abs_path.save_pretty(
            |mut file| Ok(file.write_all(template_expanded.as_bytes())?),
            overwrite,
            Some(&self.base_dir),
            cnsl,
        )
    }

    pub fn load_source(&self, problem_id: &ProblemId, cnsl: &mut Console) -> Result<String> {
        let source_abs_path = self.source_abs_path(problem_id)?;
        source_abs_path.load_pretty(
            |mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                Ok(buf)
            },
            Some(&self.base_dir),
            cnsl,
        )
    }

    pub fn exec_compile(&self, problem_id: &ProblemId) -> Result<Command> {
        let compile = &self.service().compile;
        self.exec_templ(compile, problem_id)
    }

    pub fn exec_run(&self, problem_id: &ProblemId) -> Result<Command> {
        let run = &self.service().run;
        self.exec_templ(run, problem_id)
    }

    fn problem_abs_path(&self, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        let problem_path = &self.body.problem_path;
        self.expand_to_abs(problem_path, problem_id)
    }

    pub fn testcases_abs_dir(&self, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        let testcases_dir = &self.body.testcases_dir;
        self.expand_to_abs(testcases_dir, problem_id)
    }

    fn working_abs_dir(&self, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        let working_dir = &self.service().working_dir;
        self.expand_to_abs(working_dir, problem_id)
    }

    fn source_abs_path(&self, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        let source_path = &self.service().source_path;
        self.expand_to_abs(source_path, problem_id)
    }

    fn expand_to_abs(&self, path: &TargetTempl, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        path.expand_with(self.service_id, &self.contest_id, problem_id)
            .and_then(|path_expanded| self.base_dir.join_expand(path_expanded))
    }

    fn exec_templ<'a, T: Expand<'a>>(
        &'a self,
        templ: &T,
        problem_id: &'a ProblemId,
    ) -> Result<Command>
    where
        T: Expand<'a, Context = TargetContext<'a>>,
    {
        let target_context = TargetContext::new(self.service_id, &self.contest_id, problem_id);
        let working_abs_dir = self.working_abs_dir(problem_id)?;
        let mut command = self.body.shell.exec_templ(templ, &target_context)?;
        command.current_dir(working_abs_dir.as_ref());
        Ok(command)
    }

    pub fn default_in_dir(base_dir: AbsPathBuf) -> Self {
        let body = ConfigBody::default_in_dir(&base_dir);

        Self {
            service_id: ServiceKind::default(),
            contest_id: Contest::default().id().clone(),
            base_dir,
            body,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let yaml_str = serde_yaml::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", yaml_str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigBody {
    #[serde(with = "string_serde")]
    version: Version,
    #[serde(default)]
    shell: Shell,
    #[serde(default = "ConfigBody::default_problem_path")]
    problem_path: TargetTempl,
    #[serde(default = "ConfigBody::default_testcases_dir")]
    testcases_dir: TargetTempl,
    #[serde(default)]
    session: SessionConfig,
    #[serde(default)]
    services: ServicesConfig,
}

impl ConfigBody {
    pub const FILE_NAME: &'static str = "acick.yaml";

    const DEFAULT_PROBLEM_PATH: &'static str =
        "{{ service }}/{{ contest }}/{{ problem | lower }}/problem.yaml";

    const DEFAULT_TESTCASES_DIR: &'static str =
        "{{ service }}/{{ contest }}/{{ problem | lower }}/testcases";

    pub fn generate_to(writer: &mut dyn Write) -> Result<()> {
        writeln!(
            writer,
            include_str!("../resources/acick.yaml.txt"),
            version = &*VERSION,
            bash = Shell::find_bash().display()
        )
        .context("Could not write config")
    }

    fn default_in_dir(base_dir: &AbsPathBuf) -> Self {
        Self {
            version: VERSION.clone(),
            shell: Shell::default(),
            problem_path: Self::default_problem_path(),
            testcases_dir: Self::default_testcases_dir(),
            session: SessionConfig::default_in_dir(base_dir),
            services: ServicesConfig::default(),
        }
    }

    fn default_problem_path() -> TargetTempl {
        Self::DEFAULT_PROBLEM_PATH.into()
    }

    fn default_testcases_dir() -> TargetTempl {
        Self::DEFAULT_TESTCASES_DIR.into()
    }

    fn search(cnsl: &mut Console) -> Result<AbsPathBuf> {
        let cwd = AbsPathBuf::cwd()?;
        let base_dir = cwd.search_dir_contains(Self::FILE_NAME).with_context(|| {
            format!(
                "Could not find config file ({}) in {} or any of the parent directories. \
                 Create config file first by `acick init` command.",
                Self::FILE_NAME,
                cwd
            )
        })?;
        writeln!(cnsl, "Found config file in base_dir: {}", base_dir)?;
        Ok(base_dir)
    }

    fn load(base_dir: &AbsPathBuf, cnsl: &mut Console) -> Result<Self> {
        let body: Self = base_dir.join(Self::FILE_NAME).load_pretty(
            |file| serde_yaml::from_reader(file).context("Could not read config file as yaml"),
            Some(base_dir),
            cnsl,
        )?;
        body.validate()?;
        Ok(body)
    }

    fn validate(&self) -> Result<()> {
        // check version
        let version_req = VersionReq::parse(&self.version.to_string())
            .context("Could not parse version requirement")?;
        if !version_req.matches(&VERSION) {
            return Err(anyhow!(
                r#"Found mismatched version in config file.
    config version: {}
    acick version : {}
Fix the config file so that it is compatible with the current version of acick."#,
                self.version,
                &*VERSION
            ));
        }

        Ok(())
    }
}

impl Default for ConfigBody {
    fn default() -> Self {
        Self {
            version: VERSION.clone(),
            shell: Shell::default(),
            problem_path: Self::default_problem_path(),
            testcases_dir: Self::default_testcases_dir(),
            session: SessionConfig::default(),
            services: ServicesConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct ServicesConfig {
    atcoder: ServiceConfig,
}

impl ServicesConfig {
    fn get(&self, service_id: ServiceKind) -> &ServiceConfig {
        match service_id {
            ServiceKind::Atcoder => &self.atcoder,
        }
    }
}

impl Default for ServicesConfig {
    fn default() -> Self {
        Self {
            atcoder: ServiceConfig::default_for(ServiceKind::Atcoder),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceConfig {
    lang_names: Vec<LangName>,
    working_dir: TargetTempl,
    source_path: TargetTempl,
    compile: TargetTempl,
    run: TargetTempl,
    #[serde(default)]
    template: Option<ProblemTempl>,
}

impl ServiceConfig {
    const DEFAULT_TEMPLATE: &'static str = r#"/*
[{{ contest.id }}] {{ problem.id }} - {{ problem.name }}
*/

#include <iostream>
using namespace std;

int main() {
    return 0;
}
"#;

    fn default_for(service_id: ServiceKind) -> Self {
        match service_id {
            ServiceKind::Atcoder => Self {
                lang_names: vec!["C++ (GCC 9.2.1)".into(), "C++14 (GCC 5.4.1)".into()],
                working_dir: "{{ service }}/{{ contest }}/{{ problem | lower }}".into(),
                source_path: "{{ service }}/{{ contest }}/{{ problem | lower }}/Main.cpp".into(),
                compile: "set -x && g++ -std=gnu++1y -O2 -o ./a.out ./Main.cpp".into(),
                // compile: "g++ -std=gnu++1y -O2 -I/opt/boost/gcc/include -L/opt/boost/gcc/lib -o ./a.out ./Main.cpp".into(),
                run: "./a.out".into(),
                template: Some(Self::DEFAULT_TEMPLATE.into()),
            },
        }
    }

    pub fn lang_names(&self) -> &[LangName] {
        &self.lang_names
    }
}

mod string_serde {
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
    use std::fs::OpenOptions;

    use tempfile::tempdir;

    use super::*;
    use crate::template::TargetContext;

    #[test]
    fn generate_and_deserialize() -> anyhow::Result<()> {
        let mut buf = Vec::new();
        ConfigBody::generate_to(&mut buf)?;
        let body_yaml_str = String::from_utf8(buf)?;
        let body_generated: ConfigBody = serde_yaml::from_str(&body_yaml_str)?;

        let body_default = ConfigBody::default();

        assert_eq!(body_generated, body_default);

        Ok(())
    }

    #[tokio::test]
    async fn exec_default_atcoder_compile() -> anyhow::Result<()> {
        let test_dir = tempdir()?;

        // prepare source file
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(format!("{}/Main.cpp", test_dir.path().display()))?;
        file.write_all(
            r#"
#include <iostream>
using namespace std;

int main() {{
    return 0;
}}
        "#
            .as_bytes(),
        )?;

        // exec compile command
        let contest = Contest::default();
        let problem = Problem::default();
        let shell = Shell::default();
        let compile = ServiceConfig::default_for(ServiceKind::Atcoder).compile;
        let context = TargetContext::new(ServiceKind::default(), contest.id(), problem.id());
        let output = shell
            .exec_templ(&compile, &context)?
            .current_dir(test_dir.path())
            .output()
            .await?;
        eprintln!("{:?}", output);
        assert!(output.status.success());

        Ok(())
    }
}
