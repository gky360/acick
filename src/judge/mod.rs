use std::fmt;
use std::process::{Output, Stdio};
use std::time::Duration;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::time::timeout;

use crate::model::Sample;
use crate::Result;

#[derive(
    Serialize,
    Deserialize,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Status {
    Ac,
    Wa,
    Tle,
    Re,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.into())
    }
}

pub struct Judge<'a> {
    sample: &'a Sample,
    timelimit: Duration,
}

impl<'a> Judge<'a> {
    pub fn new(sample: &'a Sample, timelimit: Duration) -> Self {
        Self { sample, timelimit }
    }

    #[tokio::main]
    pub async fn run(&self, command: Command) -> Status {
        let Self { sample, timelimit } = *self;
        let input = sample.input().as_bytes();
        let result = timeout(timelimit, Self::run_child(command, input)).await;
        match result {
            Err(_) => Status::Tle,
            // Ok(Err(err)) => Status::Re(err),
            Ok(Err(_)) => Status::Re,
            Ok(Ok(output)) => {
                if output.status.success() {
                    // TODO: check output
                    Status::Ac
                } else {
                    // Status::Re(anyhow!("{}", output.status))
                    Status::Re
                }
            }
        }
    }

    async fn run_child(mut command: Command, input: &[u8]) -> Result<Output> {
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
