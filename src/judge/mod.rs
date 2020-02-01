use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::time::{timeout, Instant};

use crate::model::Sample;
use crate::Result;

mod status;

pub use status::{Status, StatusKind};

pub struct Judge<'a> {
    sample: &'a Sample,
    time_limit: Duration,
}

impl<'a> Judge<'a> {
    pub fn new(sample: &'a Sample, time_limit: Duration) -> Self {
        Self { sample, time_limit }
    }

    #[tokio::main]
    pub async fn test(&self, command: Command) -> Status {
        let Self { sample, time_limit } = *self;
        let input = sample.input().as_bytes();

        let started_at = Instant::now();
        let result = timeout(time_limit, Self::exec_child(command, input)).await;
        let elapsed = started_at.elapsed();

        use StatusKind::*;
        match result {
            Err(_) => Status { kind: Tle, elapsed },
            Ok(Err(err)) => Status {
                kind: StatusKind::re(err),
                elapsed,
            },
            Ok(Ok(output)) => {
                if output.status.success() {
                    // TODO: check output
                    Status { kind: Ac, elapsed }
                } else {
                    Status {
                        kind: StatusKind::re(anyhow!("{}", output.status)),
                        elapsed,
                    }
                }
            }
        }
    }

    async fn exec_child(mut command: Command, input: &[u8]) -> Result<Output> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start run command")?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(input)
            .await
            .context("Could not write input to stdin")?;
        let output = child.wait_with_output().await.context("Failed to run")?;
        Ok(output)
    }
}
