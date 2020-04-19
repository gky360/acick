use std::fmt;

use anyhow::anyhow;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{with_actor, Outcome};
use crate::model::Service;
use crate::service::Act;
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct MeOpt {}

impl MeOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<MeOutcome> {
        with_actor(conf.service_id, conf.session(), |actor| {
            self.run_inner(actor, conf, cnsl)
        })
    }

    fn run_inner(&self, actor: &dyn Act, conf: &Config, cnsl: &mut Console) -> Result<MeOutcome> {
        let user = actor
            .current_user(cnsl)?
            .ok_or_else(|| anyhow!("Not logged in to {}", conf.service_id))?;

        let outcome = MeOutcome {
            service: Service::new(conf.service_id),
            username: user,
        };
        Ok(outcome)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeOutcome {
    service: Service,
    username: String,
}

impl fmt::Display for MeOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Logged in to {} as {}", self.service.id(), self.username)
    }
}

impl Outcome for MeOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;
    use crate::model::ServiceKind;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = MeOpt {};
        let outcome = run_with(&tempdir()?, |conf, cnsl| opt.run(conf, cnsl))?;
        assert_eq!(outcome.service.id(), ServiceKind::Atcoder);
        assert_eq!(&outcome.username, "acick_test");
        Ok(())
    }
}
