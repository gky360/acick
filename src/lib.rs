#![warn(clippy::all)]

#[macro_use]
extern crate strum;

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
pub struct Opt {
    #[structopt(flatten)]
    global_opt: GlobalOpt,
    #[structopt(subcommand)]
    cmd: Cmd,
}

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

impl Opt {
    pub fn run(&self) -> Result<()> {
        let conf = Config::load().context("Could not load config")?;
        let outcome = self.cmd.run(&self.global_opt, &conf)?;
        if self.global_opt.debug {
            println!("{:#?}", outcome.as_ref());
        } else {
            println!("{}", outcome.as_ref());
        }
        Ok(())
    }
}
