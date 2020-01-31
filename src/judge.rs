use std::io::Write as _;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use anyhow::Context as _;

use crate::model::Sample;
use crate::Result;

pub struct Judge<'a> {
    sample: &'a Sample,
    timelimit: Duration,
    command: Command,
}

impl<'a> Judge<'a> {
    pub fn new(sample: &'a Sample, timelimit: Duration, command: Command) -> Self {
        Self {
            sample,
            timelimit,
            command,
        }
    }

    pub fn run(self) -> Result<Output> {
        let Self {
            sample,
            timelimit,
            mut command,
        } = self;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Could not execute run command")?;
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(sample.input().as_bytes())
            .context("Could not write input to stdin of run command")?;
        let output = child
            .wait_with_output()
            .context("Failed to wait on run command")?;

        Ok(output)
    }
}
