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
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        let service_conf = conf.service();
        let lang_name = service_conf.lang_name();

        // TODO: load source
        let source = "";

        let actor = conf.build_actor();
        // TODO: receive submission
        let _submission = actor.submit(&self.problem_id, lang_name, source, cnsl)?;

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
