#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::Write;

use anyhow::Context as _;
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

use abs_path::AbsPathBuf;
use cmd::{Cmd, Run as _};
use config::Config;
use console::Console;
use model::{ContestId, ServiceKind};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

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
        let cwd = AbsPathBuf::cwd().context("Could not get current working directory")?; // TODO: search config fie
        let conf = Config::load(self.global_opt.clone(), cwd).context("Could not load config")?;
        let mut cnsl = Console::new(stderr);
        let outcome = self.cmd.run(&conf, &mut cnsl)?;

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
    use lazy_static::lazy_static;
    use reqwest::Url;

    use super::*;
    use crate::model::{Contest, Problem, Service};

    lazy_static! {
        pub static ref DEFAULT_SERVICE: Service = Service::new(ServiceKind::Atcoder);
        pub static ref DEFAULT_CONTEST: Contest =
            Contest::new("arc100", "AtCoder Regular Contest 100");
        pub static ref DEFAULT_PROBLEM: Problem = Problem::new(
            "C",
            "Linear Approximation",
            Url::parse("https://atcoder.jp/contests/arc100/tasks/arc100_a").unwrap(),
            Vec::new()
        );
    }
}
