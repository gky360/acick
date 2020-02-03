use std::{fmt, io};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Config, Console, OutputFormat, Result};

mod fetch;
mod login;
mod show;
mod submit;
mod test;

pub use fetch::FetchOpt;
pub use login::{LoginOpt, LoginOutcome};
pub use show::{ShowOpt, ShowOutcome};
pub use submit::{SubmitOpt, SubmitOutcome};
pub use test::{TestOpt, TestOutcome};

pub trait Outcome: OutcomeSerialize {
    fn is_error(&self) -> bool;
}

pub trait OutcomeSerialize: fmt::Display + fmt::Debug {
    fn write_json(&self, writer: &mut dyn io::Write) -> Result<()>;

    fn write_yaml(&self, writer: &mut dyn io::Write) -> Result<()>;

    fn print(&self, stdout: &mut dyn io::Write, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Default => writeln!(stdout, "{}", self)?,
            OutputFormat::Debug => writeln!(stdout, "{:?}", self)?,
            OutputFormat::Json => self.write_json(stdout)?,
            OutputFormat::Yaml => self.write_yaml(stdout)?,
        }
        Ok(())
    }
}

impl<T: Serialize + fmt::Display + fmt::Debug> OutcomeSerialize for T {
    fn write_json(&self, writer: &mut dyn io::Write) -> Result<()> {
        serde_json::to_writer_pretty(writer, self).context("Could not print outcome as json")
    }

    fn write_yaml(&self, writer: &mut dyn io::Write) -> Result<()> {
        serde_yaml::to_writer(writer, self).context("Could not print outcome as json")
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub enum Cmd {
    // Init(InitOpt),
    /// Shows current config
    Show(ShowOpt),
    /// Logs in to service
    #[structopt(visible_alias("l"))]
    Login(LoginOpt),
    // Participate(ParticipateOpt),
    /// Fetches problems from service
    #[structopt(visible_alias("f"))]
    Fetch(FetchOpt),
    /// Tests source code with sample inputs and outputs
    #[structopt(visible_alias("t"))]
    Test(TestOpt),
    // Judge(JudgeOpt), // test full testcases, for AtCoder only
    /// Submits source code to service
    #[structopt(visible_alias("s"))]
    Submit(SubmitOpt),
}

impl Cmd {
    pub fn run(
        &self,
        conf: &Config,
        cnsl: &mut Console,
        finish: impl FnOnce(&dyn Outcome, &mut Console) -> Result<()>,
    ) -> Result<()> {
        match self {
            Self::Show(opt) => finish(&opt.run(conf)?, cnsl),
            Self::Login(opt) => finish(&opt.run(conf, cnsl)?, cnsl),
            Self::Fetch(opt) => finish(&opt.run(conf, cnsl)?, cnsl),
            Self::Test(opt) => finish(&opt.run(conf, cnsl)?, cnsl),
            Self::Submit(opt) => finish(&opt.run(conf, cnsl)?, cnsl),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use tempfile::TempDir;

    pub fn run_with<T>(
        test_dir: &TempDir,
        run: impl FnOnce(&Config, &mut Console) -> Result<T>,
    ) -> Result<T> {
        let conf = &Config::default_test(test_dir);

        let mut output_buf = Vec::new();
        let cnsl = &mut Console::new(&mut output_buf);

        run(conf, cnsl)
    }
}
