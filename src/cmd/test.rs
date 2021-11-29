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

static DEFAULT_TIME_LIMIT_MS: u64 = 60 * 1000;

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct TestOpt {
    /// Id of the problem to be tested
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    /// If specified, uses only one sample
    sample_name: Option<String>,
    /// Tests using full testcases (only available for AtCoder)
    #[structopt(name = "full", long)]
    is_full: bool,
    /// Outpus one line per one sample
    #[structopt(long)]
    one_line: bool,
    /// Overrides time limit (in millisecs) of the problem
    #[structopt(long)]
    time_limit: Option<u64>,
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
        let time_limit = self
            .time_limit
            .map(Duration::from_millis)
            .or_else(|| problem.time_limit())
            .unwrap_or_else(|| Duration::from_millis(DEFAULT_TIME_LIMIT_MS));
        let compare = problem.compare();
        let samples = self.load_samples(problem, conf)?;
        let n_samples = samples.len();
        let max_sample_name_len = samples.max_name_len();

        if n_samples == 0 {
            return Err(anyhow!("Found no samples"));
        }

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
            if !self.one_line {
                status.describe(cnsl)?;
            }
            statuses.push(status);
        }
        let elapsed = started_at.elapsed();

        let total = TotalStatus::new(statuses);
        Ok((total, elapsed))
    }

    fn load_samples(&self, problem: Problem, conf: &Config) -> Result<Box<dyn AsSamples>> {
        if self.is_full {
            let testcases_dir = conf.testcases_abs_dir(problem.id())?;
            let testcases = AtcoderActor::load_testcases(testcases_dir, &self.sample_name)?;
            Ok(Box::new(testcases))
        } else {
            Ok(Box::new(problem.take_samples(&self.sample_name)))
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
            "{} {} {} {} ({} {}s, compile: {:.2}s, test: {:.2}s)",
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
            one_line: false,
            time_limit: None,
        };
        run_with(&test_dir, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
