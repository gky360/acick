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
        let user = ctx
            .get_env_or_prompt_stderr(user_env, "username: ", false)
            .context("Could not read username")?;
        let pass = ctx
            .get_env_or_prompt_stderr(pass_env, "password: ", true)
            .context("Could not read password")?;

        let mut service = service_id.serve(ctx);
        service.login(&user, &pass)?;

        // TODO: return outcome
        Ok(Box::new("Successfully logged in"))
    }
}

impl Default for LoginOpt {
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::cmd::tests::run_default;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        env::set_var("ACICK_ATCODER_USERNAME", "test_user");
        env::set_var("ACICK_ATCODER_PASSWORD", "test_pass");

        run_default!(LoginOpt)?;
        Ok(())
    }
}
