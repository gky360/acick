use std::fmt;
use std::io::Write as _;
use std::process::ExitStatus;

use anyhow::{anyhow, Context as _};
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::judge::{Judge, TotalStatus};
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    sample_name: Option<String>,
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
        let problem = conf
            .load_problem(&self.problem_id, cnsl)
            .context("Could not load problem file.")?;

        let compile_status = self.compile(conf)?;
        if !compile_status.success() {
            return Err(anyhow!(
                "Compile command returned non-zero status : {}",
                compile_status
            ));
        }

        let time_limit = problem.time_limit();
        let compare = problem.compare();
        let samples = if let Some(sample_name) = &self.sample_name {
            problem
                .take_samples()
                .into_iter()
                .filter(|sample| &sample.name == sample_name)
                .collect()
        } else {
            problem.take_samples()
        };
        let n_samples = samples.len();
        let mut statuses = Vec::new();
        for (i, sample) in samples.into_iter().enumerate() {
            let run = conf.exec_run(&self.problem_id)?;
            write!(
                cnsl,
                "({:>2}/{:>2}) Testing sample {} ... ",
                i + 1,
                n_samples,
                sample.name
            )?;
            let status = Judge::new(sample, time_limit, compare).test(run);
            writeln!(cnsl, "{}", status)?;
            status.describe(cnsl)?;
            statuses.push(status);
        }

        let total = TotalStatus::new(statuses);

        Ok(Box::new(TestOutcome {
            service: Service::new(conf.global_opt().service_id),
            total,
        }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestOutcome {
    service: Service,
    total: TotalStatus,
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.total)
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
            sample_name: None,
        };
        opt.run_default()?;
        Ok(())
    }
}
