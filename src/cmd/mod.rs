use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Context, Result};

mod fetch;
mod login;
mod show;

pub use fetch::FetchOpt;
pub use login::{LoginOpt, LoginOutcome};
pub use show::{ShowOpt, ShowOutcome};

pub trait Outcome: OutcomeSerialize + fmt::Display + fmt::Debug {
    fn is_error(&self) -> bool;
}

pub trait OutcomeSerialize {
    fn to_yaml(&self) -> Result<String>;
}

impl<T: Serialize> OutcomeSerialize for T {
    fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Could not serialize outcome to yaml")
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
    // New(NewOpt),
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
            let conf =
                Config::load(AbsPathBuf::cwd().expect("Could not get current working directory"))
                    .expect("Could not load config");
            let mut stdin_buf = ::std::io::BufReader::new(&b""[..]);
            let mut stderr_buf = Vec::new();
            let mut ctx = Context {
                global_opt: &global_opt,
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
