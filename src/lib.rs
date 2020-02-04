#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::Write;

use anyhow::Context as _;
use lazy_static::lazy_static;
use semver::Version;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

mod abs_path;
mod cmd;
mod config;
mod console;
mod judge;
mod model;
mod service;

use cmd::{Cmd, Outcome};
use config::Config;
use console::Console;
use model::{ContestId, ServiceKind};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

lazy_static! {
    pub static ref VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
}

#[derive(
    Serialize, EnumString, EnumVariantNames, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum OutputFormat {
    Default,
    Debug,
    Json,
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(StructOpt, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalOpt {
    #[structopt(
        name = "service",
        long,
        global = true,
        env = "ACICK_SERVICE",
        default_value = ServiceKind::default().into(),
        possible_values = &ServiceKind::VARIANTS,
    )]
    service_id: ServiceKind,
    #[structopt(
        name = "contest",
        long,
        global = true,
        env = "ACICK_CONTEST",
        default_value = "arc100"
    )]
    contest_id: ContestId,
    #[structopt(
        long,
        global = true,
        default_value = OutputFormat::default().into(),
        possible_values = &OutputFormat::VARIANTS
    )]
    output: OutputFormat,
}

impl Default for GlobalOpt {
    fn default() -> Self {
        let args = [""];
        GlobalOpt::from_iter(&args)
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
    #[structopt(flatten)]
    global_opt: GlobalOpt,
    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(&self, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
        let cnsl = &mut Console::new(stderr);

        match &self.cmd {
            Cmd::Init(opt) => self.finish(&opt.run(cnsl)?, stdout, cnsl),
            cmd => {
                let conf = &self.load_config(cnsl)?;
                match cmd {
                    Cmd::Init(_) => unreachable!(),
                    Cmd::Show(opt) => self.finish(&opt.run(conf)?, stdout, cnsl),
                    Cmd::Login(opt) => self.finish(&opt.run(conf, cnsl)?, stdout, cnsl),
                    Cmd::Fetch(opt) => self.finish(&opt.run(conf, cnsl)?, stdout, cnsl),
                    Cmd::Test(opt) => self.finish(&opt.run(conf, cnsl)?, stdout, cnsl),
                    Cmd::Submit(opt) => self.finish(&opt.run(conf, cnsl)?, stdout, cnsl),
                }
            }
        }
    }

    fn load_config(&self, cnsl: &mut Console) -> Result<Config> {
        let service_id = self.global_opt.service_id;
        let contest_id = &self.global_opt.contest_id;
        Config::load(service_id, contest_id.clone(), cnsl).context("Could not load config")
    }

    fn finish(
        &self,
        outcome: &dyn Outcome,
        stdout: &mut dyn Write,
        cnsl: &mut Console,
    ) -> Result<()> {
        cnsl.flush()?;
        writeln!(stdout)?;

        outcome.print(stdout, self.global_opt.output)?;

        if outcome.is_error() {
            Err(Error::msg("Command exited with error"))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lazy_static::lazy_static;

    use super::*;
    use crate::model::{Compare, Contest, Problem, Service};

    lazy_static! {
        pub static ref DEFAULT_SERVICE: Service = Service::new(ServiceKind::Atcoder);
        pub static ref DEFAULT_CONTEST: Contest =
            Contest::new("arc100", "AtCoder Regular Contest 100");
        pub static ref DEFAULT_PROBLEM: Problem = Problem::new(
            "C",
            "Linear Approximation",
            "arc100_a",
            Duration::from_secs(2),
            "1024 MB".parse().unwrap(),
            Compare::Default,
            Vec::new()
        );
    }
}
