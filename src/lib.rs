#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use anyhow::Context as _;
use rpassword::read_password_from_tty;
use std::{env, fmt, io};
use structopt::StructOpt;
use strum::VariantNames;

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

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalOpt {
    #[structopt(
        name = "service",
        long,
        global = true,
        env = "ACICK_SERVICE",
        default_value = ServiceKind::Atcoder.into(),
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
    #[structopt(long, global = true)]
    debug: bool,
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
        mut stdin: impl io::BufRead + fmt::Debug,
        mut stdout: impl io::Write,
        mut stderr: impl io::Write + fmt::Debug,
    ) -> Result<()> {
        let cwd = AbsPathBuf::cwd().context("Could not get current working directory")?; // TODO: search config fie
        let conf = Config::load(cwd).context("Could not load config")?;
        let mut ctx = Context {
            global_opt: &self.global_opt,
            conf: &conf,
            stdin: &mut stdin,
            stderr: &mut stderr,
        };
        let outcome = self.cmd.run(&mut ctx)?;

        ctx.stderr.flush()?;
        writeln!(stdout)?;

        if self.global_opt.debug {
            writeln!(stdout, "{:#?}", &outcome)
        } else {
            writeln!(stdout, "{}", &outcome)
        }?;

        if outcome.is_error() {
            Err(Error::msg("Command exited with error"))
        } else {
            Ok(())
        }
    }
}

pub trait Input: io::BufRead + fmt::Debug {
    fn read_input(&mut self, is_password: bool) -> Result<String> {
        let raw = if is_password {
            read_password_from_tty(None)?
        } else {
            let mut buf = String::new();
            self.read_line(&mut buf)?;
            buf
        };
        Ok(raw.trim().to_string())
    }
}

impl<T: io::BufRead + fmt::Debug> Input for T {}

pub trait Output: io::Write + fmt::Debug {
    fn prompt(&mut self, prompt: &str) -> Result<()> {
        write!(self, "{}", prompt)?;
        self.flush()?;
        Ok(())
    }
}

impl<T: io::Write + fmt::Debug> Output for T {}

#[derive(Debug)]
pub struct Context<'a> {
    global_opt: &'a GlobalOpt,
    conf: &'a Config,
    stdin: &'a mut dyn Input,
    stderr: &'a mut dyn Output,
}

impl Context<'_> {
    fn prompt_read(&mut self, prompt: &str, is_password: bool) -> Result<String> {
        self.stderr
            .prompt(prompt)
            .context("Could not output prompt message")?;
        self.stdin.read_input(is_password)
    }

    fn get_env_or_prompt_read(
        &mut self,
        env_name: &str,
        prompt: &str,
        is_password: bool,
    ) -> Result<String> {
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
        self.prompt_read(prompt, is_password)
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::*;
    use crate::model::{Contest, Problem, Service};

    lazy_static! {
        pub static ref DEFAULT_SERVICE: Service = Service::new(ServiceKind::Atcoder);
        pub static ref DEFAULT_CONTEST: Contest =
            Contest::new("arc100", "AtCoder Regular Contest 100", Vec::new());
        pub static ref DEFAULT_PROBLEM: Problem =
            Problem::new("C", "Linear Approximation", Vec::new());
    }
}
