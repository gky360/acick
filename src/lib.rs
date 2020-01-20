#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::env;
use std::io;

use anyhow::Context as _;
use rpassword::read_password_from_tty;
use structopt::StructOpt;
use strum::VariantNames;

mod cmd;
mod config;
mod model;
mod service;

use cmd::{Cmd, Run as _};
use config::Config;
use model::ServiceKind;

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
        default_value = "abc100"
    )]
    contest_id: String,
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
    pub fn run<I: Input, O: Output, E: Output>(&self, ctx: &mut Context<I, O, E>) -> Result<()> {
        let conf = Config::load().context("Could not load config")?;
        let outcome = self.cmd.run(&self.global_opt, &conf, ctx)?;

        ctx.stdout.flush()?;
        ctx.stderr.flush()?;
        if self.global_opt.debug {
            writeln!(ctx.stdout, "\n{:#?}", outcome.as_ref())
        } else {
            writeln!(ctx.stdout, "\n{}", outcome.as_ref())
        }?;
        Ok(())
    }
}

pub trait Input: io::BufRead {
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

impl<T: io::BufRead> Input for T {}

pub trait Output: io::Write {}

impl<T: io::Write> Output for io::BufWriter<T> {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context<I: Input, O: Output, E: Output> {
    stdin: I,
    stdout: O,
    stderr: E,
}

impl<I: Input, O: Output, E: Output> Context<I, O, E> {
    pub fn new(stdin: I, stdout: O, stderr: E) -> Self {
        Self {
            stdin,
            stdout,
            stderr,
        }
    }

    fn prompt_stderr(&mut self, prompt: &str, is_password: bool) -> Result<String> {
        write!(self.stderr, "{}", prompt)?;
        self.stderr.flush()?;
        self.stdin.read_input(is_password)
    }

    fn get_env_or_prompt_stderr(
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
        self.prompt_stderr(prompt, is_password)
    }
}

impl<'a>
    Context<io::StdinLock<'a>, io::BufWriter<io::StdoutLock<'a>>, io::BufWriter<io::StderrLock<'a>>>
{
    pub fn from_stdio(
        stdin: &'a io::Stdin,
        stdout: &'a io::Stdout,
        stderr: &'a io::Stderr,
    ) -> Self {
        Self {
            stdin: stdin.lock(),
            stdout: io::BufWriter::new(stdout.lock()),
            stderr: io::BufWriter::new(stderr.lock()),
        }
    }
}
