use std::{fmt, io};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;

use crate::abs_path::AbsPathBuf;
use crate::config::SessionConfig;
use crate::model::{ContestId, ServiceKind, DEFAULT_CONTEST_ID_STR};
use crate::service::act::Act;
use crate::{Config, Console, OutputFormat, Result};

mod fetch;
mod init;
mod login;
mod logout;
mod me;
mod show;
mod submit;
mod test;

pub use fetch::FetchOpt;
pub use init::{InitOpt, InitOutcome};
pub use login::{LoginOpt, LoginOutcome};
pub use logout::{LogoutOpt, LogoutOutcome};
pub use me::{MeOpt, MeOutcome};
pub use show::{ShowOpt, ShowOutcome};
pub use submit::{SubmitOpt, SubmitOutcome};
pub use test::{TestOpt, TestOutcome};

use crate::atcoder::AtcoderActor;

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
    /// Gets info of user currently logged in to service
    Me {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: MeOpt,
    },
    /// Logs in to service
    #[structopt(visible_alias("l"))]
    Login {
        #[structopt(flatten)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: LoginOpt,
    },
    /// Logs out from all services
    Logout {
        #[structopt(skip)]
        sc: ServiceContest,
        #[structopt(flatten)]
        opt: LogoutOpt,
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
        base_dir: Option<AbsPathBuf>,
        cnsl: &mut Console,
        finish: impl FnOnce(&dyn Outcome, &mut Console) -> Result<()>,
    ) -> Result<()> {
        let b = base_dir;
        match self {
            Self::Init(opt) => finish(&opt.run(b, cnsl)?, cnsl),
            Self::Show { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?)?, cnsl),
            Self::Me { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
            Self::Login { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
            Self::Logout { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
            Self::Fetch { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
            Self::Test { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
            Self::Submit { sc, opt } => finish(&opt.run(&sc.load_config(b, cnsl)?, cnsl)?, cnsl),
        }
    }
}

#[derive(Default, StructOpt, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceContest {
    #[structopt(
        name = "service",
        long,
        short,
        global = true,
        env = "ACICK_SERVICE",
        default_value = ServiceKind::default().into(),
        possible_values = &ServiceKind::VARIANTS,
    )]
    pub service_id: ServiceKind,
    #[structopt(
        name = "contest",
        long,
        short,
        global = true,
        env = "ACICK_CONTEST",
        default_value = DEFAULT_CONTEST_ID_STR,
    )]
    pub contest_id: ContestId,
}

impl ServiceContest {
    fn load_config(&self, base_dir: Option<AbsPathBuf>, cnsl: &mut Console) -> Result<Config> {
        Config::load(self.service_id, self.contest_id.clone(), base_dir, cnsl)
            .context("Could not load config file")
    }
}

fn with_actor<F, R>(service_id: ServiceKind, session: &SessionConfig, f: F) -> R
where
    F: FnOnce(&dyn Act) -> R,
{
    match service_id {
        ServiceKind::Atcoder => f(&AtcoderActor::new(session)),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::abs_path::AbsPathBuf;
    use crate::{Config, Console, ConsoleConfig};

    use tempfile::TempDir;

    pub fn run_with<T>(
        test_dir: &TempDir,
        run: impl FnOnce(&Config, &mut Console) -> Result<T>,
    ) -> Result<T> {
        eprintln!("{}", std::env::current_dir()?.display());

        let base_dir = AbsPathBuf::try_new(test_dir.path().to_owned()).unwrap();
        let conf = Config::default_in_dir(base_dir);
        let mut cnsl = Console::buf(ConsoleConfig { assume_yes: true });
        let result = run(&conf, &mut cnsl);

        let output_str = cnsl.take_output()?;
        eprintln!("{}", output_str);
        result
    }
}
