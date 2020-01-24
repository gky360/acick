use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Context, Result};

mod login;
mod show;

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
    /// Shows current config
    Show(ShowOpt),
    /// Log in to service
    Login(LoginOpt),
    // Participate(ParticipateOpt),
    // New(NewOpt),
    // Get(GetOpt),
    // Judge(JudgeOpt),
    // Submit(SubmitOpt),
}

impl Run for Cmd {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        match self {
            Self::Show(opt) => opt.run(ctx),
            Self::Login(opt) => opt.run(ctx),
        }
    }
}

#[cfg(test)]
mod tests {
    macro_rules! run_default {
        ($opt:ident) => {{
            use crate::{Config, GlobalOpt};

            let opt = $opt::default();
            let global_opt = GlobalOpt::default();
            let conf = Config::load()?;
            let mut stdin_buf = ::std::io::BufReader::new(&b""[..]);
            let mut stderr_buf = Vec::new();
            let mut ctx = Context {
                global_opt: &global_opt,
                conf: &conf,
                stdin: &mut stdin_buf,
                stderr: &mut stderr_buf,
            };

            opt.run(&mut ctx)
        }};
    }
    pub(crate) use run_default;
}
