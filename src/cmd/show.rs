use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::{Config, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl ShowOpt {
    pub fn run<'a>(&self, conf: &'a Config) -> Result<ShowOutcome<'a>> {
        Ok(ShowOutcome(conf))
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShowOutcome<'a>(&'a Config);

impl fmt::Display for ShowOutcome<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Outcome for ShowOutcome<'_> {
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
        run_with(&tempdir()?, |conf, _| opt.run(conf).map(|_| ()))?;
        Ok(())
    }
}
