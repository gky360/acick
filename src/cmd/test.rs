use std::fmt;
use std::process::ExitStatus;

use anyhow::{anyhow, Context as _};
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::judge::Judge;
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
}

impl TestOpt {
    #[tokio::main]
    async fn compile(&self, conf: &Config) -> Result<ExitStatus> {
        let mut compile = conf.exec_compile(&self.problem_id)?;
        Ok(compile.status().await?)
    }
}

impl Run for TestOpt {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        let problem = conf.load_problem(&self.problem_id, cnsl)
            .context("Could not load problem file. \
            Make sure the problem id is correct and the problem file is created by `fetch` command.")?;
        eprintln!("{:?}", problem);

        let compile_status = self.compile(conf)?;
        if !compile_status.success() {
            return Err(anyhow!(
                "Compile command returned non-zero status : {}",
                compile_status
            ));
        }

        for sample in problem.samples() {
            let run = conf.exec_run(&self.problem_id)?;
            let judge = Judge::new(sample, std::time::Duration::from_secs(1));
            let status = judge.run(run);
            eprintln!("{}", status);
        }

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

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = crate::cmd::fetch::FetchOpt::default();
        opt.run_default()?;

        let opt = TestOpt {
            problem_id: "c".into(),
        };
        opt.run_default()?;
        Ok(())
    }
}
