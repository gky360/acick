use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::{Context, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct ShowOpt {}

impl Run for ShowOpt {
    fn run(&self, ctx: &mut Context) -> Result<Box<dyn Outcome>> {
        Ok(Box::new(ctx.conf.clone()))
    }
}

impl Default for ShowOpt {
    fn default() -> Self {
        Self {}
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
