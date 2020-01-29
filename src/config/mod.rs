use std::fmt;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use dirs::{data_local_dir, home_dir};
use getset::{CopyGetters, Getters};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;
use semver::Version;
use serde::{Deserialize, Serialize};

mod template;

use crate::abs_path::{AbsPathBuf, ToAbs as _};
use crate::model::{string, Contest, Problem, Service, ServiceKind};
use crate::service::{AtcoderService, CookieStorage, Serve};
use crate::{Console, GlobalOpt, Result};
use template::{Expand as _, ProblemContext, ProblemTempl, Shell, TemplArray};

#[derive(Serialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
pub struct Config {
    global_opt: GlobalOpt,
    base_dir: AbsPathBuf,
    body: ConfigBody,
}

impl Config {
    pub fn load(global_opt: GlobalOpt, base_dir: AbsPathBuf) -> Result<Self> {
        // TODO: load from file
        let body = ConfigBody::default();
        let version_str = body.version.to_string();
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
            Ok(Self {
                global_opt,
                base_dir,
                body,
            })
        }
    }

    pub fn build_service<'a>(&'a self) -> Box<dyn Serve + 'a> {
        let client = self
            .get_client_builder()
            .build()
            .expect("Could not setup client. \
                TLS backend cannot be initialized, or the resolver cannot load the system configuration.");
        let service_id = self.global_opt.service_id;
        match service_id {
            ServiceKind::Atcoder => Box::new(AtcoderService::new(client, self)),
        }
    }

    fn get_client_builder(&self) -> ClientBuilder {
        let session = &self.body.session;
        let user_agent = &session.user_agent;
        let timeout = session.timeout;
        // TODO : switch client by service
        Client::builder()
            .referer(false)
            .redirect(Policy::none()) // redirects manually
            .user_agent(user_agent)
            .timeout(Some(timeout))
    }

    pub fn open_cookie_storage(&self) -> Result<CookieStorage> {
        let cookies_path = &self.body.session.cookies_path;
        CookieStorage::open(&cookies_path.to_abs(&self.base_dir))
    }

    pub fn save_problem(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
        overwrite: bool,
        cnsl: &mut Console,
    ) -> Result<bool> {
        let problem_abs_path = self.problem_abs_path(service, contest, problem)?;
        problem_abs_path.save_pretty(
            &self.base_dir,
            overwrite,
            |file| serde_yaml::to_writer(file, &problem).context("Could not save problem as yaml"),
            cnsl,
        )
    }

    pub fn expand_and_save_source(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
        overwrite: bool,
        cnsl: &mut Console,
    ) -> Result<bool> {
        let source_abs_path = self.source_abs_path(service, contest, problem)?;
        let template = &self.body.services.get(service.id()).template;
        let template_expanded = template.expand_with(service, contest, problem)?;
        source_abs_path.save_pretty(
            &self.base_dir,
            overwrite,
            |mut file| Ok(file.write_all(template_expanded.as_bytes())?),
            cnsl,
        )
    }

    fn problem_abs_path(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
    ) -> Result<AbsPathBuf> {
        let problem_context = ProblemContext::new(service, contest, problem);
        let problem_path = &self.body.problem_path;
        let problem_path_expanded = problem_path.expand(&problem_context)?;
        Ok(self.base_dir.join(problem_path_expanded))
    }

    fn source_abs_path(
        &self,
        service: &Service,
        contest: &Contest,
        problem: &Problem,
    ) -> Result<AbsPathBuf> {
        let problem_context = ProblemContext::new(service, contest, problem);
        let source_path = &self.body.services.get(service.id()).source_path;
        let source_path_expanded = source_path.expand(&problem_context)?;
        Ok(self.base_dir.join(source_path_expanded))
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
pub struct ConfigBody {
    #[serde(with = "string")]
    #[get = "pub"]
    version: Version,
    #[get = "pub"]
    shell: Shell,
    problem_path: ProblemTempl,
    #[get = "pub"]
    session: SessionConfig,
    #[get = "pub"]
    services: ServicesConfig,
}

impl Default for ConfigBody {
    fn default() -> Self {
        Self {
            version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            shell: Shell::default(),
            problem_path:
                "/tmp/acick/{{ service.id }}/{{ contest.id }}/{{ problem.id | lower }}/problem.yaml"
                    .into(),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct ServicesConfig {
    atcoder: ServiceConfig,
}

impl ServicesConfig {
    pub fn get(&self, service_id: ServiceKind) -> &ServiceConfig {
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
    language: String,
    working_dir: ProblemTempl,
    source_path: ProblemTempl,
    compile: TemplArray<ProblemTempl>,
    run: TemplArray<ProblemTempl>,
    template: ProblemTempl,
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
                language: "C++14 (GCC 5.4.1)".into(),
                working_dir:
                    "/tmp/acick/{{ service.id }}/{{ contest.id }}/{{ problem.id | lower }}".into(),
                source_path:
                    "/tmp/acick/{{ service.id }}/{{ contest.id }}/{{ problem.id | lower }}/Main.cpp"
                        .into(),
                compile: (&[
                    "g++",
                    "-std=gnu++1y",
                    "-O2",
                    "-I/opt/boost/gcc/include",
                    "-L/opt/boost/gcc/lib",
                    "-o",
                    "./a.out",
                    "./Main.cpp",
                ])
                    .into(),
                run: (&["./a.out"]).into(),
                template: Self::DEFAULT_TEMPLATE.into(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::template::ProblemContext;
    use crate::tests::{DEFAULT_CONTEST, DEFAULT_PROBLEM, DEFAULT_SERVICE};

    #[test]
    fn serialize_default() -> anyhow::Result<()> {
        serde_yaml::to_string(&Config::load(GlobalOpt::default(), AbsPathBuf::cwd()?)?)?;
        Ok(())
    }

    #[test]
    fn exec_default_atcoder_compile() -> anyhow::Result<()> {
        let shell = Shell::default();
        let compile = ServiceConfig::default_for(ServiceKind::Atcoder).compile;
        let context = ProblemContext::new(&DEFAULT_SERVICE, &DEFAULT_CONTEST, &DEFAULT_PROBLEM);
        let output = shell.exec_templ_arr(&compile, &context)?;
        println!("{:?}", output);
        // TODO: assert success
        Ok(())
    }
}
