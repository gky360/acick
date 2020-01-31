use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
}

#[cfg(test)]
impl Default for TestOpt {
    fn default() -> Self {
        Self {
            problem_id: "c".into(),
        }
    }
}

impl Run for TestOpt {
    fn run(&self, conf: &Config, _cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        let mut compile = conf.exec_compile(&self.problem_id)?;
        eprintln!("{:?}", compile.output());
        let mut run = conf.exec_run(&self.problem_id)?;
        eprintln!("{:?}", run.output());

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
