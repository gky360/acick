use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::{Config, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl ShowOpt {
    pub fn run(&self, conf: &Config) -> Result<ShowOutcome> {
        Ok(ShowOutcome { conf: conf.clone() })
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
    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let opt = ShowOpt {};
        run_with(&tempdir()?, |conf, _| opt.run(conf))?;
        Ok(())
    }
}
