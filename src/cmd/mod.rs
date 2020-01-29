use std::{fmt, io};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Context, OutputFormat, Result};

mod fetch;
mod login;
mod show;

pub use fetch::FetchOpt;
pub use login::{LoginOpt, LoginOutcome};
pub use show::{ShowOpt, ShowOutcome};

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
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>>;
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
    // Test(TestOpt), // test samples
    // Judge(JudgeOpt), // test full testcases, for AtCoder only
    // Submit(SubmitOpt),
}

impl Run for Cmd {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        match self {
            Self::Show(opt) => opt.run(ctx),
            Self::Login(opt) => opt.run(ctx),
            Self::Fetch(opt) => opt.run(ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    macro_rules! run_default {
        ($opt:ident) => {{
            use crate::abs_path::AbsPathBuf;
            use crate::{Config, GlobalOpt};

            let opt = $opt::default();
            let global_opt = GlobalOpt::default();
            let conf = Config::load(
                global_opt,
                AbsPathBuf::cwd().expect("Could not get current working directory"),
            )
            .expect("Could not load config");
            let mut stdin_buf = ::std::io::BufReader::new(&b""[..]);
            let mut stderr_buf = Vec::new();
            let mut ctx = Context {
                conf: &conf,
                stdin: &mut stdin_buf,
                stderr: &mut stderr_buf,
            };

            let result = opt.run(&mut ctx);
            eprintln!("{}", String::from_utf8_lossy(&stderr_buf));
            result
        }};
    }
    pub(crate) use run_default;
}
