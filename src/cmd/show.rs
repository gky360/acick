use std::fmt;

use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::{Config, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(StructOpt, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(ShowOutcome {
            conf: ctx.conf.clone(),
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShowOutcome {
    pub conf: Config,
}

impl fmt::Display for ShowOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.conf.fmt(f)
    }
}

impl Outcome for ShowOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::tests::run_default;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        run_default!(ShowOpt)?;
        Ok(())
    }
}
