use std::fmt;

use anyhow::{anyhow, Context as _};
use serde::Serialize;
use structopt::StructOpt;

use crate::abs_path::AbsPathBuf;
use crate::cmd::Outcome;
use crate::config::ConfigBody;
use crate::{Console, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct InitOpt {
    /// Overwrites config file if exists
    #[structopt(long, short = "w")]
    overwrite: bool,
}

impl InitOpt {
    pub fn run(&self, base_dir: Option<AbsPathBuf>, cnsl: &mut Console) -> Result<InitOutcome> {
        // decide base_dir
        let cwd = AbsPathBuf::cwd()?;
        let base_dir = base_dir.unwrap_or_else(|| cwd.clone());

        // check if base_dir exists
        if !base_dir.as_ref().is_dir() {
            return Err(anyhow!("Could not find directory : {}", base_dir));
        }

        // save config to yaml file
        let config_path = base_dir.join(ConfigBody::FILE_NAME);
        let is_saved = config_path.save_pretty(
            |mut file| ConfigBody::generate_to(&mut file).context("Could not save config"),
            self.overwrite,
            Some(&cwd),
            cnsl,
        )?;

        // check if saved
        if is_saved == None {
            return Err(anyhow!("Config file already exists : {}", config_path));
        }

        Ok(InitOutcome { config_path })
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InitOutcome {
    config_path: AbsPathBuf,
}

impl fmt::Display for InitOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Saved config file : {}", self.config_path)
    }
}

impl Outcome for InitOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::ConsoleConfig;

    #[test]
    fn run_default() -> anyhow::Result<()> {
        let cnsl = &mut Console::buf(ConsoleConfig::default());

        let test_dir = tempdir()?;
        let opt = InitOpt { overwrite: false };
        let base_dir = AbsPathBuf::try_new(test_dir.path())?;
        opt.run(Some(base_dir), cnsl)?;
        Ok(())
    }
}
