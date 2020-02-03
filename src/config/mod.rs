use std::fmt;
use std::io::{Read as _, Write as _};

use anyhow::{anyhow, Context as _};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

mod session_config;
mod template;

use crate::abs_path::AbsPathBuf;
use crate::model::{
    string, Contest, ContestId, LangName, LangNameRef, Problem, ProblemId, Service, ServiceKind,
};
use crate::service::{Act, AtcoderActor};
use crate::{Console, Result, VERSION};
pub use session_config::SessionConfig;
use template::{Expand, ProblemTempl, Shell, TargetContext, TargetTempl, TemplArray};

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    pub service_id: ServiceKind,
    pub contest_id: ContestId,
    base_dir: AbsPathBuf,
    body: ConfigBody,
}

impl Config {
    pub fn load(
        service_id: ServiceKind,
        contest_id: ContestId,
        cnsl: &mut Console,
    ) -> Result<Self> {
        let (body, base_dir) = ConfigBody::search(cnsl)?;
        Ok(Self {
            service_id,
            contest_id,
            base_dir,
            body,
        })
    }

    pub fn service(&self) -> &ServiceConfig {
        self.body.services.get(self.service_id)
    }

    pub fn build_actor<'a>(&'a self) -> Box<dyn Act + 'a> {
        let client = self.body.session
            .get_client_builder()
            .build()
            .expect("Could not setup client. \
                TLS backend cannot be initialized, or the resolver cannot load the system configuration.");
        match self.service_id {
            ServiceKind::Atcoder => Box::new(AtcoderActor::new(client, &self.body.session)),
        }
    }

    pub fn save_problem(
        &self,
        problem: &Problem,
        overwrite: bool,
        cnsl: &mut Console,
    ) -> Result<bool> {
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
        let problem: Problem = problem_abs_path.load_pretty(
            |file| serde_yaml::from_reader(file).context("Could not read problem as yaml"),
            Some(&self.base_dir),
            cnsl,
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
    ) -> Result<bool> {
        if service.id() != self.service_id || contest.id() != &self.contest_id {
            return Err(anyhow!("Found mismatching service id or contest id"));
        }
        let source_abs_path = self.source_abs_path(problem.id())?;
        let template = match &self.service().template {
            Some(template) => template,
            None => return Ok(false), // skip if template is empty
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
        self.exec_templ_arr(compile, problem_id)
    }

    pub fn exec_run(&self, problem_id: &ProblemId) -> Result<Command> {
        let run = &self.service().run;
        self.exec_templ_arr(run, problem_id)
    }

    fn problem_abs_path(&self, problem_id: &ProblemId) -> Result<AbsPathBuf> {
        let problem_path = &self.body.problem_path;
        self.expand_to_abs(problem_path, problem_id)
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

    fn exec_templ_arr<'a, T>(
        &'a self,
        templ_arr: &TemplArray<T>,
        problem_id: &'a ProblemId,
    ) -> Result<Command>
    where
        T: Expand<'a, Context = TargetContext<'a>>,
    {
        let target_context = TargetContext::new(self.service_id, &self.contest_id, problem_id);
        let working_abs_dir = self.working_abs_dir(problem_id)?;
        let mut command = self.body.shell.exec_templ_arr(templ_arr, &target_context)?;
        command.current_dir(working_abs_dir.as_ref());
        Ok(command)
    }
}

#[cfg(test)]
impl Config {
    pub fn default_test() -> Self {
        use crate::GlobalOpt;

        let GlobalOpt {
            service_id,
            contest_id,
            ..
        } = GlobalOpt::default();
        Self {
            service_id,
            contest_id,
            base_dir: AbsPathBuf::try_new(std::env::temp_dir().join(env!("CARGO_PKG_NAME")))
                .unwrap(),
            body: ConfigBody::default(),
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
    #[serde(with = "string")]
    version: Version,
    #[serde(default)]
    shell: Shell,
    #[serde(default = "ConfigBody::default_problem_path")]
    problem_path: TargetTempl,
    #[serde(default)]
    session: SessionConfig,
    #[serde(default)]
    services: ServicesConfig,
}

impl ConfigBody {
    pub const FILE_NAME: &'static str = "acick.yaml";

    const DEFAULT_PROBLEM_PATH: &'static str =
        "{{ service }}/{{ contest }}/{{ problem | lower }}/problem.yaml";

    fn default_problem_path() -> TargetTempl {
        Self::DEFAULT_PROBLEM_PATH.into()
    }

    fn search(cnsl: &mut Console) -> Result<(Self, AbsPathBuf)> {
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
        Ok((Self::load(&base_dir, cnsl)?, base_dir))
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
        let version_req =
            VersionReq::parse(&self.version.to_string()).context("Could not parse version")?;
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
            problem_path: Self::DEFAULT_PROBLEM_PATH.into(),
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
    lang_name: LangName,
    working_dir: TargetTempl,
    source_path: TargetTempl,
    compile: TemplArray<TargetTempl>,
    run: TemplArray<TargetTempl>,
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
                lang_name: "C++14 (GCC 5.4.1)".into(),
                working_dir: "{{ service }}/{{ contest }}/{{ problem | lower }}".into(),
                source_path: "{{ service }}/{{ contest }}/{{ problem | lower }}/Main.cpp".into(),
                compile: (&[
                    "g++",
                    "-std=gnu++1y",
                    "-O2",
                    // "-I/opt/boost/gcc/include",
                    // "-L/opt/boost/gcc/lib",
                    "-o",
                    "./a.out",
                    "./Main.cpp",
                ])
                    .into(),
                run: (&["./a.out"]).into(),
                template: Some(Self::DEFAULT_TEMPLATE.into()),
            },
        }
    }

    pub fn lang_name(&self) -> LangNameRef {
        &self.lang_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abs_path::AbsPathBuf;
    use crate::config::template::TargetContext;
    use crate::tests::{DEFAULT_CONTEST, DEFAULT_PROBLEM, DEFAULT_SERVICE};

    #[test]
    fn serialize_default() -> anyhow::Result<()> {
        serde_yaml::to_string(&Config::default_test())?;
        Ok(())
    }

    #[test]
    fn deserialize_example() -> anyhow::Result<()> {
        let mut output_buf = Vec::new();
        let cnsl = &mut Console::new(&mut output_buf);

        let default_body = ConfigBody::default();
        let mut example_body = ConfigBody::load(&AbsPathBuf::cwd()?, cnsl)?;
        // ignore difference on shell because it varies depending on environments
        example_body.shell = Shell::default();

        eprintln!("{}", String::from_utf8_lossy(&output_buf));

        assert_eq!(example_body, default_body);
        Ok(())
    }

    #[test]
    fn exec_default_atcoder_compile() -> anyhow::Result<()> {
        let shell = Shell::default();
        let compile = ServiceConfig::default_for(ServiceKind::Atcoder).compile;
        let context = TargetContext::new(
            DEFAULT_SERVICE.id(),
            &DEFAULT_CONTEST.id(),
            &DEFAULT_PROBLEM.id(),
        );
        let output = shell.exec_templ_arr(&compile, &context)?;
        println!("{:?}", output);
        // TODO: assert success
        Ok(())
    }
}
