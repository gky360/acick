use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self, conf: &Config, _cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(ShowOutcome { conf: conf.clone() }))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShowOutcome {
    conf: Config,
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

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = ShowOpt {};
        opt.run_default()?;
        Ok(())
    }
}
