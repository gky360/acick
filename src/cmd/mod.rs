use std::{fmt, io};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Config, Console, OutputFormat, Result};

mod fetch;
mod login;
mod show;
mod test;

pub use fetch::FetchOpt;
pub use login::{LoginOpt, LoginOutcome};
pub use show::{ShowOpt, ShowOutcome};
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

pub trait Run {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>>;

    #[cfg(test)]
    fn run_default(&self) -> Result<Box<dyn Outcome>> {
        let conf = Config::default();

        let mut output_buf = Vec::new();
        let cnsl = &mut Console::new(&mut output_buf);

        let result = self.run(&conf, cnsl);
        eprintln!("{}", String::from_utf8_lossy(&output_buf));
        result
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub enum Cmd {
    // Init(InitOpt),
    /// Shows current config
    Show(ShowOpt),
    /// Logs in to service
    Login(LoginOpt),
    // Participate(ParticipateOpt),
    /// Fetches problems from service
    Fetch(FetchOpt),
    /// Tests source code with sample inputs and outputs
    Test(TestOpt),
    // Judge(JudgeOpt), // test full testcases, for AtCoder only
    // Submit(SubmitOpt),
}

impl Run for Cmd {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        match self {
            Self::Show(opt) => opt.run(conf, cnsl),
            Self::Login(opt) => opt.run(conf, cnsl),
            Self::Fetch(opt) => opt.run(conf, cnsl),
            Self::Test(opt) => opt.run(conf, cnsl),
        }
    }
}
