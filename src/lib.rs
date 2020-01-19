#![warn(clippy::all)]

#[macro_use]
extern crate strum;

use structopt::StructOpt;
use strum::VariantNames;

mod cmd;
mod config;
mod model;

use cmd::Cmd;
use config::Config;
use model::ServiceKind;

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Opt {
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

    #[structopt(subcommand)]
    cmd: Cmd,
}

impl Opt {
    pub fn run(&self) -> Result<()> {
        eprintln!("{:?}", self);
        let conf = Config::load();
        eprintln!("{}", serde_yaml::to_string(&conf)?);
        println!("Hello, world!");
        Ok(())
    }
}
