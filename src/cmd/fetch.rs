use std::fmt;

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;

use crate::abs_path::AbsPathBuf;
use crate::cmd::Outcome;
use crate::dropbox::{
    DbxAuthorizer, DBX_APP_KEY, DBX_APP_SECRET, DBX_REDIRECT_PATH, DBX_REDIRECT_PORT,
};
use crate::model::{Contest, Problem, ProblemId, Service, ServiceKind};
use crate::service::AtcoderActor;
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
    /// Opens problems in browser
    #[structopt(name = "open", long, short)]
    need_open: bool,
    /// Fetches full testcases from dropbox (only available for AtCoder)
    #[structopt(name = "full", long)]
    is_full: bool,
}

#[cfg(test)]
impl FetchOpt {
    pub fn default_test() -> Self {
        Self {
            problem_id: None,
            overwrite: false,
            need_open: false,
            is_full: false,
        }
    }
}

impl FetchOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<FetchOutcome> {
        let Self {
            ref problem_id,
            overwrite,
            need_open,
            is_full,
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

        // open problem in browser if needed
        if need_open {
            for problem in problems.iter() {
                actor.open_problem_url(&conf.contest_id, problem, cnsl)?;
            }
        }

        if is_full {
            if conf.service_id == ServiceKind::Atcoder {
                // TODO: load paths from config
                let token_path = AbsPathBuf::try_new("/tmp/acick/token.json".into())?;
                let test_cases_path = AbsPathBuf::try_new("/tmp/acick/testcases".into())?;

                // authorize Dropbox account
                let dropbox = DbxAuthorizer::new(
                    DBX_APP_KEY,
                    DBX_APP_SECRET,
                    DBX_REDIRECT_PORT,
                    DBX_REDIRECT_PATH,
                    &token_path,
                )
                .load_or_request(cnsl)?;

                AtcoderActor::fetch_full(
                    &dropbox,
                    &conf.contest_id,
                    &problems,
                    &test_cases_path,
                    cnsl,
                )?;
            } else {
                cnsl.warn("\"--full\" option is only available for AtCoder")?;
            }
        }

        Ok(FetchOutcome {
            service,
            contest,
            problems,
        })
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FetchOutcome {
    service: Service,
    contest: Contest,
    problems: Vec<Problem>,
}

impl fmt::Display for FetchOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.problems.is_empty() {
            write!(f, "Found no problems")
        } else if self.problems.len() == 1 {
            write!(f, "Successfully fetched 1 problem")
        } else {
            write!(f, "Successfully fetched {} problems", self.problems.len())
        }
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
