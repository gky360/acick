use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct SubmitOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
}

impl Run for SubmitOpt {
    fn run(&self, conf: &Config, _cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(SubmitOutcome {
            service: Service::new(conf.global_opt().service_id),
        }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubmitOutcome {
    service: Service,
}

impl fmt::Display for SubmitOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This is submit outcome")
    }
}

impl Outcome for SubmitOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn run_default() -> anyhow::Result<()> {
        let opt = SubmitOpt { problem_id: "c".into() };
        opt.run_default()?;
        Ok(())
    }
}
