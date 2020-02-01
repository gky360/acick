use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use tokio::io::{AsyncWriteExt as _, BufWriter};
use tokio::process::Command;
use tokio::time::{timeout, Instant};

use crate::model::{Compare, Sample};
use crate::Result;

mod diff;
mod status;

use diff::TextDiff;
pub use status::{Status, StatusKind, TotalStatus};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Judge {
    sample: Sample,
    time_limit: Duration,
    cmp: Compare,
}

impl Judge {
    pub fn new(sample: Sample, time_limit: Duration, cmp: Compare) -> Self {
        Self {
            sample,
            time_limit,
            cmp,
        }
    }

    #[tokio::main]
    pub async fn test(self, command: Command) -> Status {
        let Self {
            sample,
            time_limit,
            cmp,
        } = self;
        let sample_name = sample.name;
        let input = sample.input.as_bytes();

        let started_at = Instant::now();
        let result = timeout(time_limit, Self::exec_child(command, input)).await;
        let elapsed = started_at.elapsed();

        match result {
            Err(_) => Status::tle(sample_name, elapsed),
            Ok(Err(err)) => Status::re(sample_name, elapsed, err),
            Ok(Ok(output)) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let diff = TextDiff::new("expected", "actual", stdout, sample.output, cmp);
                if diff.is_any() {
                    Status::wa(sample_name, elapsed, diff)
                } else {
                    Status::ac(sample_name, elapsed, diff)
                }
            }
            Ok(Ok(output)) => Status::re(sample_name, elapsed, anyhow!("{}", output.status)),
        }
    }

    async fn exec_child(mut command: Command, input: &[u8]) -> Result<Output> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start run command")?;
        let mut stdin = BufWriter::new(child.stdin.as_mut().unwrap());
        stdin
            .write_all(input)
            .await
            .context("Could not write input to stdin")?;
        stdin.flush().await.context("Could not flush stdin")?;
        let output = child.wait_with_output().await.context("Failed to run")?;
        Ok(output)
    }
}
