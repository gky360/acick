#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use std::io;

use anyhow::Context as _;
use structopt::StructOpt;
use strum::VariantNames;

mod cmd;
mod config;
mod model;

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
        if self.global_opt.debug {
            writeln!(ctx.stdout, "{:#?}", outcome.as_ref())
        } else {
            writeln!(ctx.stdout, "{}", outcome.as_ref())
        }?;
        Ok(())
    }
}

pub trait Input: io::BufRead {}

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
}
