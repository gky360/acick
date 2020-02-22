use std::io;
use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use tokio::io::{AsyncWriteExt as _, BufWriter};
use tokio::process::Command;
use tokio::time::{timeout, Instant};

use crate::model::{AsSample, Compare};
use crate::Result;

mod diff;
mod status;

use diff::TextDiff;
pub use status::{Status, StatusKind, TotalStatus};

#[derive(Debug)]
pub struct Judge<'a> {
    sample: &'a dyn AsSample,
    time_limit: Duration,
    cmp: Compare,
}

impl<'a> Judge<'a> {
    pub fn new(sample: &'a dyn AsSample, time_limit: Duration, cmp: Compare) -> Self {
        Self {
            sample,
            time_limit,
            cmp,
        }
    }

    pub async fn test(&self, command: Command) -> Result<Status> {
        let Self {
            sample,
            time_limit,
            cmp,
        } = *self;
        let sample_name = sample.name().to_owned();
        let input = sample.input();

        let started_at = Instant::now();
        let result = timeout(time_limit, Self::exec_child(command, input)).await;
        let elapsed = started_at.elapsed();

        match result {
            Err(_) => Ok(Status::tle(sample_name, elapsed)),
            Ok(Err(err)) => Err(err),
            Ok(Ok(output)) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let diff = TextDiff::new("expected", "actual", stdout, sample.output_str()?, cmp);
                if diff.is_any() {
                    Ok(Status::wa(sample_name, elapsed, diff))
                } else {
                    Ok(Status::ac(sample_name, elapsed, diff))
                }
            }
            Ok(Ok(output)) => Ok(Status::re(
                sample_name,
                elapsed,
                anyhow!("{}", output.status),
            )),
        }
    }

    async fn exec_child(mut command: Command, mut input: io::Cursor<&[u8]>) -> Result<Output> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start run command")?;
        let mut stdin = BufWriter::new(child.stdin.as_mut().unwrap());

        // async write to stdin may cause broken pipe error
        // when write is performed after the child exited
        Self::ignore_broken_pipe(tokio::io::copy(&mut input, &mut stdin).await.map(|_| ()))
            .context("Could not write input to stdin")?;
        Self::ignore_broken_pipe(stdin.flush().await).context("Could not flush stdin")?;

        let output = child.wait_with_output().await.context("Failed to run")?;
        Ok(output)
    }

    fn ignore_broken_pipe(
        result: std::result::Result<(), io::Error>,
    ) -> std::result::Result<(), io::Error> {
        result.or_else(|err| match err.kind() {
            io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(err),
        })
    }
}
