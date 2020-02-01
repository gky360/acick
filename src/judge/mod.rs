use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::{anyhow, Context as _};
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::time::{timeout, Instant};

use crate::model::{Compare, Sample};
use crate::Result;

mod diff;
mod status;

use diff::TextDiff;
pub use status::{Status, StatusKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Judge<'a> {
    sample: &'a Sample,
    time_limit: Duration,
    cmp: Compare,
}

impl<'a> Judge<'a> {
    pub fn new(sample: &'a Sample, time_limit: Duration, cmp: Compare) -> Self {
        Self {
            sample,
            time_limit,
            cmp,
        }
    }

    #[tokio::main]
    pub async fn test(&self, command: Command) -> Status {
        let Self {
            sample,
            time_limit,
            cmp,
        } = *self;
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
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let diff = TextDiff::new(&stdout, &sample.output(), cmp);
                    if diff.is_any() {
                        Status { kind: Wa, elapsed }
                    } else {
                        Status { kind: Ac, elapsed }
                    }
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
