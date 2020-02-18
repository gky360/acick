use std::fmt;
use std::io::Write as _;

use anyhow::{anyhow, Context as _};
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::judge::{Judge, StatusKind, TotalStatus};
use crate::model::{Problem, ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    sample_name: Option<String>,
}

impl TestOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<TestOutcome> {
        // load problem file
        let problem = conf
            .load_problem(&self.problem_id, cnsl)
            .context("Could not load problem file.")?;

        let total = self.compile_and_test(problem, conf, cnsl)?;

        // build output
        Ok(TestOutcome {
            service: Service::new(conf.service_id),
            total,
        })
    }

    async fn compile(&self, conf: &Config) -> Result<()> {
        let mut compile = conf.exec_compile(&self.problem_id)?;
        let exit_status = compile.status().await?;
        if !exit_status.success() {
            return Err(anyhow!(
                "Compile command returned non-zero status : {}",
                exit_status
            ));
        }
        Ok(())
    }

    async fn test(
        &self,
        problem: Problem,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<TotalStatus> {
        let time_limit = problem.time_limit();
        let compare = problem.compare();
        let samples = problem.take_samples(&self.sample_name);

        // test source code with samples
        let n_samples = samples.len();
        let mut statuses = Vec::new();
        writeln!(cnsl)?;
        for (i, sample) in samples.into_iter().enumerate() {
            let run = conf.exec_run(&self.problem_id)?;
            write!(
                cnsl,
                "[{:>2}/{:>2}] Testing sample {} ... ",
                i + 1,
                n_samples,
                sample.name
            )?;
            let status = Judge::new(sample, time_limit, compare).test(run).await;
            writeln!(cnsl, "{}", status)?;
            status.describe(cnsl)?;
            statuses.push(status);
        }

        let total = TotalStatus::new(statuses);
        Ok(total)
    }

    #[tokio::main]
    async fn compile_and_test(
        &self,
        problem: Problem,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<TotalStatus> {
        self.compile(conf).await.context("Failed to compile")?;
        self.test(problem, conf, cnsl).await
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
        self.total.kind() != StatusKind::Ac
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let test_dir = tempdir()?;

        let fetch_opt = crate::cmd::FetchOpt::default_test();
        run_with(&test_dir, |conf, cnsl| fetch_opt.run(conf, cnsl))?;

        let opt = TestOpt {
            problem_id: "c".into(),
            sample_name: None,
        };
        run_with(&test_dir, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
