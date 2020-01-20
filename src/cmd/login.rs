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
        global_opt: &GlobalOpt,
        _conf: &Config,
        ctx: &mut Context<I, O, E>,
    ) -> Result<Box<dyn Outcome>> {
        let GlobalOpt { service_id, .. } = global_opt;
        let (user_env, pass_env) = service_id.to_user_pass_env_names();
        let username = ctx
            .get_env_or_prompt_stderr(user_env, "username: ", false)
            .context("Could not read username")?;
        let password = ctx
            .get_env_or_prompt_stderr(pass_env, "password: ", true)
            .context("Could not read password")?;
        eprintln!("{:?}", (username, password));

        // TODO: return outcome
        Ok(Box::new("Successfully logged in"))
    }
}
