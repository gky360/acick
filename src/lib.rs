#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io::{self, Read, Write};
use std::{env, fmt};

use anyhow::Context as _;
use serde::Serialize;
use structopt::StructOpt;
use strum::VariantNames;
use termion::input::TermRead as _;

mod abs_path;
mod cmd;
mod config;
mod model;
mod service;

use abs_path::AbsPathBuf;
use cmd::{Cmd, Run as _};
use config::Config;
use model::{ContestId, ServiceKind};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

#[derive(
    Serialize, EnumString, EnumVariantNames, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum OutputFormat {
    Default,
    Debug,
    Json,
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(StructOpt, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalOpt {
    #[structopt(
        name = "service",
        long,
        global = true,
        env = "ACICK_SERVICE",
        default_value = ServiceKind::default().into(),
        possible_values = &ServiceKind::VARIANTS,
    )]
    service_id: ServiceKind,
    #[structopt(
        name = "contest",
        long,
        global = true,
        env = "ACICK_CONTEST",
        default_value = "arc100"
    )]
    contest_id: ContestId,
    #[structopt(
        long,
        global = true,
        default_value = OutputFormat::default().into(),
        possible_values = &OutputFormat::VARIANTS
    )]
    output: OutputFormat,
}

impl Default for GlobalOpt {
    fn default() -> Self {
        let args = [""];
        GlobalOpt::from_iter(&args)
    }
}

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
    #[structopt(flatten)]
    global_opt: GlobalOpt,
    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(
        &self,
        stdin: &mut dyn Read,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
    ) -> Result<()> {
        let cwd = AbsPathBuf::cwd().context("Could not get current working directory")?; // TODO: search config fie
        let conf = Config::load(self.global_opt.clone(), cwd).context("Could not load config")?;
        let mut cnsl = Console { stdin, stderr };
        let outcome = self.cmd.run(&conf, &mut cnsl)?;

        cnsl.stderr.flush()?;
        writeln!(stdout)?;

        outcome.print(stdout, self.global_opt.output)?;

        if outcome.is_error() {
            Err(Error::msg("Command exited with error"))
        } else {
            Ok(())
        }
    }
}

pub struct Console<'a> {
    stdin: &'a mut dyn Read,
    stderr: &'a mut dyn Write,
}

impl fmt::Debug for Console<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Console")
    }
}

impl Console<'_> {
    fn read_user(&mut self, is_password: bool) -> io::Result<String> {
        if is_password {
            self.stdin.read_passwd(&mut self.stderr)
        } else {
            self.read_line()
        }
        .and_then(|maybe_str| {
            maybe_str.ok_or_else(|| io::Error::new(io::ErrorKind::Interrupted, "Interrupted"))
        })
    }

    fn prompt(&mut self, prompt: &str) -> io::Result<()> {
        write!(self, "{}", prompt)?;
        self.flush()?;
        Ok(())
    }

    fn prompt_and_read(&mut self, prompt: &str, is_password: bool) -> io::Result<String> {
        self.prompt(prompt)?;
        self.read_user(is_password)
    }

    fn get_env_or_prompt_and_read(
        &mut self,
        env_name: &str,
        prompt: &str,
        is_password: bool,
    ) -> io::Result<String> {
        if let Ok(val) = env::var(env_name) {
            writeln!(
                self.stderr,
                "{}{:16} (read from env {})",
                prompt,
                if is_password { "********" } else { &val },
                env_name
            )?;
            return Ok(val);
        };
        self.prompt_and_read(prompt, is_password)
    }
}

impl Read for Console<'_> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for Console<'_> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use reqwest::Url;

    use super::*;
    use crate::model::{Contest, Problem, Service};

    lazy_static! {
        pub static ref DEFAULT_SERVICE: Service = Service::new(ServiceKind::Atcoder);
        pub static ref DEFAULT_CONTEST: Contest =
            Contest::new("arc100", "AtCoder Regular Contest 100", Vec::new());
        pub static ref DEFAULT_PROBLEM: Problem = Problem::new(
            "C",
            "Linear Approximation",
            Url::parse("https://atcoder.jp/contests/arc100/tasks/arc100_a").unwrap(),
            Vec::new()
        );
    }
}
