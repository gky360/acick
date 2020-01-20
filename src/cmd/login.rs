use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::config::Config;
use crate::{GlobalOpt, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl Run for LoginOpt {
    fn run(&self, _global_opt: &GlobalOpt, _conf: &Config) -> Result<Box<dyn Outcome>> {
        // TODO: impl
        Ok(Box::new(""))
    }
}
