#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::Write;

use lazy_static::lazy_static;
use semver::Version;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

mod abs_path;
mod cmd;
pub mod config;
mod console;
mod judge;
mod model;
mod service;

use cmd::{Cmd, Outcome};
use config::Config;
use console::Console;
use model::{Contest, Service, ServiceKind};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

lazy_static! {
    static ref VERSION: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    static ref DEFAULT_SERVICE: Service = Service::new(ServiceKind::Atcoder);
    static ref DEFAULT_CONTEST: Contest = Contest::new("arc100", "AtCoder Regular Contest 100");
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

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
    #[structopt(
        long,
        global = true,
        default_value = OutputFormat::default().into(),
        possible_values = &OutputFormat::VARIANTS
    )]
    output: OutputFormat,
    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(&self, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<()> {
        let cnsl = &mut Console::new(stderr);
        self.cmd
            .run(cnsl, |outcome, cnsl| self.finish(outcome, stdout, cnsl))
    }

    fn finish(
        &self,
        outcome: &dyn Outcome,
        stdout: &mut dyn Write,
        cnsl: &mut Console,
    ) -> Result<()> {
        cnsl.flush()?;
        writeln!(stdout)?;

        outcome.print(stdout, self.output)?;

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

    use crate::model::{Compare, Problem};

    lazy_static! {
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
