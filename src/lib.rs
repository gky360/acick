#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::{self, Write};

use lazy_static::lazy_static;
use semver::Version;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

mod abs_path;
mod cmd;
pub mod config;
mod console;
mod dropbox;
mod judge;
mod macros;
mod model;
mod service;

use crate::cmd::{Cmd, Outcome};
use crate::config::Config;
use crate::console::{Console, ConsoleConfig};
use crate::model::{Contest, Service, ServiceKind};

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
    /// Format of output
    #[structopt(
        long,
        global = true,
        default_value = OutputFormat::default().into(),
        possible_values = &OutputFormat::VARIANTS
    )]
    output: OutputFormat,
    /// Hides any messages except the final outcome of commands
    #[structopt(long, short, global = true)]
    quiet: bool,
    /// Assumes "yes" as answer to all prompts and run non-interactively
    #[structopt(long, short = "y", global = true)]
    assume_yes: bool,
    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(&self) -> Result<()> {
        let assume_yes = self.assume_yes;
        let cnsl_conf = ConsoleConfig { assume_yes };
        let mut cnsl = if self.quiet {
            Console::sink(cnsl_conf)
        } else {
            Console::term(cnsl_conf)
        };

        self.cmd.run(&mut cnsl, |outcome, cnsl| {
            self.finish(outcome, &mut io::stdout(), cnsl)
        })
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
