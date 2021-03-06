#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::{self, Write};
use std::path::PathBuf;

use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

use acick_atcoder as atcoder;
use acick_config as config;
use acick_util::{abs_path, console, model, service, DATA_LOCAL_DIR};

mod cmd;
mod judge;

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
    /// Sets path to the directory that contains a config file
    #[structopt(long, short, global = true)]
    base_dir: Option<PathBuf>,
    /// Specifies the format of output
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

        let base_dir = match &self.base_dir {
            Some(base_dir) => Some(abs_path::AbsPathBuf::cwd()?.join(base_dir)),
            None => None,
        };
        self.cmd.run(base_dir, &mut cnsl, |outcome, cnsl| {
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
        if self.quiet {
            stdout.flush()?;
        } else {
            writeln!(stdout)?;
        }

        outcome.print(stdout, self.output)?;

        if outcome.is_error() {
            Err(Error::msg("Command exited with error"))
        } else {
            Ok(())
        }
    }
}
