use std::{fmt, io};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

use crate::model::{ContestId, ServiceKind};
use crate::{Config, Console, OutputFormat, Result, DEFAULT_CONTEST, DEFAULT_SERVICE};

mod fetch;
mod init;
mod login;
mod show;
mod submit;
mod test;

pub use fetch::FetchOpt;
pub use init::{InitOpt, InitOutcome};
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
    /// Creates config file
    Init(InitOpt),
    /// Shows current config
    Show {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: ShowOpt,
    },
    /// Logs in to service
    #[structopt(visible_alias("l"))]
    Login {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: LoginOpt,
    },
    // Participate(ParticipateOpt),
    /// Fetches problems from service
    #[structopt(visible_alias("f"))]
    Fetch {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: FetchOpt,
    },
    /// Tests source code with sample inputs and outputs
    #[structopt(visible_alias("t"))]
    Test {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: TestOpt,
    },
    // Judge(JudgeOpt), // test full testcases, for AtCoder only
    /// Submits source code to service
    #[structopt(visible_alias("s"))]
    Submit {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: SubmitOpt,
    },
}

impl Cmd {
    pub fn run(
        &self,
        cnsl: &mut Console,
        finish: impl FnOnce(&dyn Outcome, &mut Console) -> Result<()>,
    ) -> Result<()> {
        match self {
            Self::Init(opt) => finish(&opt.run(cnsl)?, cnsl),
            Self::Show { sc, opt } => finish(&opt.run(&sc.load_config(cnsl)?)?, cnsl),
            Self::Login { sc, opt } => finish(&opt.run(&sc.load_config(cnsl)?, cnsl)?, cnsl),
            Self::Fetch { sc, opt } => finish(&opt.run(&sc.load_config(cnsl)?, cnsl)?, cnsl),
            Self::Test { sc, opt } => finish(&opt.run(&sc.load_config(cnsl)?, cnsl)?, cnsl),
            Self::Submit { sc, opt } => finish(&opt.run(&sc.load_config(cnsl)?, cnsl)?, cnsl),
        }
    }
}

#[derive(StructOpt, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceContest {
    #[structopt(
        name = "service",
        long,
        short,
        global = true,
        env = "ACICK_SERVICE",
        default_value = DEFAULT_SERVICE.id().into(),
        possible_values = &ServiceKind::VARIANTS,
    )]
    pub service_id: ServiceKind,
    #[structopt(
        name = "contest",
        long,
        short,
        global = true,
        env = "ACICK_CONTEST",
        default_value = DEFAULT_CONTEST.id().as_ref(),
    )]
    pub contest_id: ContestId,
}

impl ServiceContest {
    fn load_config(&self, cnsl: &mut Console) -> Result<Config> {
        Config::load(self.service_id, self.contest_id.clone(), cnsl)
            .context("Could not load config file")
    }
}

impl Default for ServiceContest {
    fn default() -> Self {
        Self {
            service_id: DEFAULT_SERVICE.id(),
            contest_id: DEFAULT_CONTEST.id().clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{Config, Console, ConsoleConfig};

    use tempfile::TempDir;

    pub fn run_with<T>(
        test_dir: &TempDir,
        run: impl FnOnce(&Config, &mut Console) -> Result<T>,
    ) -> Result<T> {
        eprintln!("{}", std::env::current_dir()?.display());

        let conf = Config::default_test(test_dir);
        let mut cnsl = Console::buf(ConsoleConfig::default());
        let result = run(&conf, &mut cnsl);

        let output_str = String::from_utf8(cnsl.take_buf().unwrap())?;
        eprintln!("{}", output_str);
        result
    }
}
