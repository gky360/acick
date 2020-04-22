use std::fmt;

use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::{Config, Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct LogoutOpt {}

impl LogoutOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<LogoutOutcome> {
        let cookies_path = conf.session().cookies_path();
        cookies_path.remove_file_pretty(Some(&conf.base_dir), cnsl)?;
        Ok(LogoutOutcome {})
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogoutOutcome {}

impl fmt::Display for LogoutOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Successfully logged out from all services")
    }
}

impl Outcome for LogoutOutcome {
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
        let opt = LogoutOpt {};
        run_with(&tempdir()?, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
