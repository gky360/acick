use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::Service;
use crate::{Config, Console, Result};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {}

impl Run for TestOpt {
    fn run(&self, conf: &Config, _cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(TestOutcome {
            service: Service::new(conf.global_opt().service_id),
        }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestOutcome {
    service: Service,
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl Outcome for TestOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::tests::run_default;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        run_default!(TestOpt)?;
        Ok(())
    }
}
