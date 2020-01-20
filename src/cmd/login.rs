use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::config::Config;
use crate::{Context, GlobalOpt, Input, Output, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LoginOpt {}

impl Run for LoginOpt {
    fn run<I: Input, O: Output, E: Output>(
        &self,
        _global_opt: &GlobalOpt,
        _conf: &Config,
        _ctx: &mut Context<I, O, E>,
    ) -> Result<Box<dyn Outcome>> {
        // TODO: impl
        Ok(Box::new(""))
    }
}
