#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::{self, Write};

use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

use acick_config as config;
use acick_dropbox as dropbox;
use acick_util::{abs_path, console, model, service, web};

mod cmd;
mod judge;
mod service_old;

use crate::cmd::{Cmd, Outcome};
use crate::config::Config;
use crate::console::{Console, ConsoleConfig};

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
