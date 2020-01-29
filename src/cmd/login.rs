use std::fmt;
use std::io::Write as _;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::Service;
use crate::{Config, Console, GlobalOpt, Result};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl Run for LoginOpt {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        let GlobalOpt { service_id, .. } = conf.global_opt();
        let (user_env, pass_env) = service_id.to_user_pass_env_names();
        let user = cnsl
            .get_env_or_prompt_and_read(user_env, "username: ", false)
            .context("Could not read username")?;
        let pass = cnsl
            .get_env_or_prompt_and_read(pass_env, "password: ", true)
            .context("Could not read password")?;
        writeln!(cnsl)?;

        let service = conf.build_service();
        let is_not_already = service.login(user.to_owned(), pass, cnsl)?;

        let outcome = LoginOutcome {
            service: Service::new(conf.global_opt().service_id),
            username: user,
            is_already: !is_not_already,
        };
        Ok(Box::new(outcome))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginOutcome {
    service: Service,
    username: String,
    is_already: bool,
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
            self.service.id(),
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
