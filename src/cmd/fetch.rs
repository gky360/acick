use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::model::{Contest, ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct FetchOpt {
    /// If specified, fetches only one problem
    #[structopt(name = "problem")]
    problem_id: Option<ProblemId>,
    /// Overwrites existing problem files and source files
    #[structopt(long, short = "w")]
    overwrite: bool,
}

#[cfg(test)]
impl FetchOpt {
    pub fn default_test() -> Self {
        Self {
            problem_id: None,
            overwrite: true,
        }
    }
}

impl FetchOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<FetchOutcome> {
        let Self {
            ref problem_id,
            overwrite,
        } = *self;

        // fetch data from service
        let actor = conf.build_actor();
        let (contest, problems) = actor.fetch(&conf.contest_id, problem_id, cnsl)?;

        let service = Service::new(conf.service_id);

        // save problem data file
        for problem in problems.iter() {
            conf.save_problem(problem, overwrite, cnsl)
                .context("Could not save problem data file")?;
        }

        // expand source template and save source file
        for problem in problems.iter() {
            conf.expand_and_save_source(&service, &contest, problem, overwrite, cnsl)
                .context("Could not save source file from template")?;
        }

        Ok(FetchOutcome { service, contest })
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
    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = FetchOpt::default_test();
        run_with(&tempdir()?, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
