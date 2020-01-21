use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::{Context, Result};

mod login;
mod show;

use login::LoginOpt;
use show::ShowOpt;

pub trait Outcome: fmt::Display + fmt::Debug {
    fn to_yaml(&self) -> Result<String>;
}

impl<T: fmt::Display + fmt::Debug + Serialize> Outcome for T {
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
            let (stdin, stderr) = (::std::io::stdin(), ::std::io::stderr());
            let mut ctx = Context {
                global_opt: &global_opt,
                conf: &conf,
                stdin: &mut stdin.lock(),
                stderr: &mut stderr.lock(),
            };

            opt.run(&mut ctx)
        }};
    }
    pub(crate) use run_default;
}
