use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{Contest, ProblemId};
use crate::{Context, GlobalOpt, Result};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct FetchOpt {
    /// Problem id. If specified, only one problem will be fetched.
    #[structopt(name = "problem")]
    problem_id: Option<ProblemId>,
}

impl Run for FetchOpt {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        let GlobalOpt { service_id, .. } = ctx.global_opt;
        let mut service = service_id.serve(ctx);
        let contest = service.fetch(&self.problem_id)?;
        Ok(Box::new(FetchOutcome { contest }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FetchOutcome {
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
