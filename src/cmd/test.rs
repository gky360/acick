use std::fmt;
use std::io::Write as _;
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use serde::Serialize;
use structopt::StructOpt;
use tokio::time::Instant;

use crate::atcoder::AtcoderActor;
use crate::cmd::Outcome;
use crate::judge::{Judge, StatusKind, TotalStatus};
use crate::model::{AsSamples, ContestId, Problem, ProblemId, Service};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    sample_name: Option<String>,
    #[structopt(name = "full", long)]
    is_full: bool,
}

fn testcase_or_sample(is_full: bool) -> &'static str {
    if is_full {
        "testcase"
    } else {
        "sample"
    }
}

impl TestOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<TestOutcome> {
        let problem = conf.load_problem(&self.problem_id, cnsl)?;
        let problem_name = problem.name().to_owned();

        let (total, compile_elapsed, test_elapsed) = self.compile_and_test(problem, conf, cnsl)?;

        // build output
        Ok(TestOutcome {
            service: Service::new(conf.service_id),
            contest_id: conf.contest_id.to_owned(),
            problem_id: self.problem_id.to_owned(),
            problem_name,
            total,
            compile_elapsed,
            test_elapsed,
            is_full: self.is_full,
        })
    }

    async fn compile(&self, conf: &Config) -> Result<Duration> {
        let started_at = Instant::now();
        let mut compile = conf.exec_compile(&self.problem_id)?;
        let exit_status = compile.status().await?;
        let elapsed = started_at.elapsed();

        if !exit_status.success() {
            return Err(anyhow!(
                "Compile command returned non-zero status : {}",
                exit_status
            ));
        }
        Ok(elapsed)
    }

    async fn test(
        &self,
        problem: Problem,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<(TotalStatus, Duration)> {
        let time_limit = problem.time_limit();
        let compare = problem.compare();
        let samples = self.load_samples(problem, conf)?;
        let n_samples = samples.len();
        let max_sample_name_len = samples.max_name_len();

        // test source code with samples
        let started_at = Instant::now();
        let mut statuses = Vec::new();
        writeln!(cnsl)?;
        for (i, sample) in samples.enumerate() {
            let sample = sample?;
            let run = conf.exec_run(&self.problem_id)?;
            write!(
                cnsl,
                "[{:>2}/{:>2}] {} {:>l$} ... ",
                i + 1,
                n_samples,
                testcase_or_sample(self.is_full),
                sample.name(),
                l = max_sample_name_len,
            )?;
            let status = Judge::new(sample, time_limit, compare).test(run).await?;
            writeln!(cnsl, "{}", status)?;
            status.describe(cnsl)?;
            statuses.push(status);
        }
        let elapsed = started_at.elapsed();

        let total = TotalStatus::new(statuses);
        Ok((total, elapsed))
    }

    fn load_samples<'a>(&'a self, problem: Problem, conf: &Config) -> Result<Box<dyn AsSamples>> {
        if self.is_full {
            let testcases_dir = conf.testcases_abs_dir(problem.id())?;
            let testcases = AtcoderActor::load_testcases(testcases_dir, &self.sample_name)?;
            Ok(Box::new(testcases))
        } else {
            Ok(Box::new(problem.take_samples()))
        }
    }

    #[tokio::main]
    async fn compile_and_test(
        &self,
        problem: Problem,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<(TotalStatus, Duration, Duration)> {
        let compile_elapsed = self.compile(conf).await.context("Failed to compile")?;
        let (total, test_elapsed) = self.test(problem, conf, cnsl).await?;
        Ok((total, compile_elapsed, test_elapsed))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TestOutcome {
    service: Service,
    contest_id: ContestId,
    problem_id: ProblemId,
    problem_name: String,
    total: TotalStatus,
    compile_elapsed: Duration,
    test_elapsed: Duration,
    is_full: bool,
}

impl fmt::Display for TestOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Tested {} {} {} {} ({} {}s, compile: {:.2}s, test: {:.2}s)",
            self.service.id(),
            self.contest_id,
            self.problem_id,
            self.problem_name,
            self.total.count(),
            testcase_or_sample(self.is_full),
            (self.compile_elapsed.as_secs_f32()),
            (self.test_elapsed.as_secs_f32()),
        )?;
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
            is_full: false,
        };
        run_with(&test_dir, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
