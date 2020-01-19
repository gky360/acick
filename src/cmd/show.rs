use anyhow::Context as _;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::config::Config;
use crate::Result;

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self) -> Result<Box<dyn Outcome>> {
        let outcome = Config::load().context("Could not load config")?;
        Ok(Box::new(outcome))
    }
}
