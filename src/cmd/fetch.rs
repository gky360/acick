use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{Contest, ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct FetchOpt {
    /// If specified, fetches only one problem
    #[structopt(name = "problem")]
    problem_id: Option<ProblemId>,
    /// Overwrites existing problem files and source files
    #[structopt(long, short = "w")]
    overwrite: bool,
}

impl Run for FetchOpt {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        let service = conf.build_service();
        let contest = service.fetch(&self.problem_id, cnsl)?;

        conf.save_problems_files(&contest, self.overwrite, cnsl)?;

        Ok(Box::new(FetchOutcome {
            service: Service::new(conf.global_opt().service_id),
            contest,
        }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FetchOutcome {
    service: Service,
    contest: Contest,
}

impl fmt::Display for FetchOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Successfully fetched problems")
    }
}

impl Outcome for FetchOutcome {
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
        run_default!(FetchOpt)?;
        Ok(())
    }
}
