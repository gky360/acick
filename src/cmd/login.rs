use std::fmt;
use std::io::Write as _;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{with_actor, Outcome};
use crate::model::Service;
use crate::service::Act;
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl LoginOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<LoginOutcome> {
        with_actor(conf.service_id, conf.session(), |actor| {
            self.run_inner(actor, conf, cnsl)
        })
    }

    fn run_inner(
        &self,
        actor: &dyn Act,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<LoginOutcome> {
        let (user_env, pass_env) = conf.service_id.to_user_pass_env_names();
        let user = cnsl
            .get_env_or_prompt_and_read(user_env, "username: ", false)
            .context("Could not read username")?;
        let pass = cnsl
            .get_env_or_prompt_and_read(pass_env, "password: ", true)
            .context("Could not read password")?;
        writeln!(cnsl)?;

        let is_not_already = actor.login(user.to_owned(), pass, cnsl)?;

        let outcome = LoginOutcome {
            service: Service::new(conf.service_id),
            username: user,
            is_already: !is_not_already,
        };
        Ok(outcome)
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

    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;

    #[test]
    fn check_envs_for_user_and_pass() -> anyhow::Result<()> {
        assert!(!env::var("ACICK_ATCODER_USERNAME")?.is_empty());
        assert!(!env::var("ACICK_ATCODER_PASSWORD")?.is_empty());
        Ok(())
    }

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = LoginOpt {};
        run_with(&tempdir()?, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
