use anyhow::Context as _;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::config::Config;
use crate::{Context, GlobalOpt, Input, Output, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl Run for LoginOpt {
    fn run<I: Input, O: Output, E: Output>(
        &self,
        _global_opt: &GlobalOpt,
        _conf: &Config,
        ctx: &mut Context<I, O, E>,
    ) -> Result<Box<dyn Outcome>> {
        let username = ctx
            .prompt_stderr("username: ", false)
            .context("Could not read username")?;
        let password = ctx
            .prompt_stderr("password: ", true)
            .context("Could not read password")?;
        eprintln!("{}", username);
        eprintln!("{}", password);

        // TODO: return outcome
        Ok(Box::new("Successfully logged in"))
    }
}
