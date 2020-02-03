use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Error, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct SubmitOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
}

impl Run for SubmitOpt {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        // load problem file
        let problem = conf
            .load_problem(&self.problem_id, cnsl)
            .context("Could not load problem file.")?;

        // load source
        let source = conf
            .load_source(&self.problem_id, cnsl)
            .context("Could not load source file")?;
        if source.is_empty() {
            return Err(Error::msg("Found empty source file"));
        }

        // submit
        let actor = conf.build_actor();
        let lang_name = conf.service().lang_name();
        actor.submit(&problem, lang_name, &source, cnsl)?;

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
        let opt = SubmitOpt {
            problem_id: "c".into(),
        };
        opt.run_default()?;
        Ok(())
    }
}
