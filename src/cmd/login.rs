use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::ServiceKind;
use crate::{Context, GlobalOpt, Result};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl Run for LoginOpt {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        let GlobalOpt { service_id, .. } = ctx.conf.global_opt();
        let (user_env, pass_env) = service_id.to_user_pass_env_names();
        let user = ctx
            .get_env_or_prompt_read(user_env, "username: ", false)
            .context("Could not read username")?;
        let pass = ctx
            .get_env_or_prompt_read(pass_env, "password: ", true)
            .context("Could not read password")?;
        writeln!(ctx.stderr)?;

        let mut service = service_id.serve(ctx);
        let outcome = service.login(user, pass)?;

        Ok(Box::new(outcome))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginOutcome {
    pub service_id: ServiceKind,
    pub username: String,
    pub is_already: bool,
}

impl fmt::Display for LoginOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} logged in to {} as {}",
            if self.is_already {
                "Already"
            } else {
                "Successfully"
            },
            self.service_id,
            &self.username
        )
    }
}

impl Outcome for LoginOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::cmd::tests::run_default;

    fn check_envs_for_user_and_pass() -> anyhow::Result<()> {
        assert!(!env::var("ACICK_ATCODER_USERNAME")?.is_empty());
        assert!(!env::var("ACICK_ATCODER_PASSWORD")?.is_empty());
        Ok(())
    }

    #[test]
    fn run_default() -> anyhow::Result<()> {
        check_envs_for_user_and_pass()?;
        run_default!(LoginOpt)?;
        Ok(())
    }
}
