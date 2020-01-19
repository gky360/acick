use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::config::Config;
use crate::{GlobalOpt, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self, _global_opt: &GlobalOpt, conf: &Config) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(conf.clone()))
    }
}
