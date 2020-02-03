use std::fmt;

use anyhow::Context as _;
use chrono::{offset::Local, DateTime, SecondsFormat};
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::{Outcome, Run};
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Error, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct SubmitOpt {
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    #[structopt(long, short)]
    force: bool,
}

impl Run for SubmitOpt {
    fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<Box<dyn Outcome>> {
        // confirm
        let contest_id = &conf.global_opt().contest_id;
        let message = format!("submit problem {} to {}?", &self.problem_id, contest_id);
        if !self.force && !cnsl.confirm(&message, false)? {
            return Err(Error::msg("Not submitted"));
        }

        // load problem file
        let problem = conf
            .load_problem(&self.problem_id, cnsl)
            .context("Could not load problem file.")?;

        // load source
        let source = conf
            .load_source(&self.problem_id, cnsl)
            .context("Could not load source file")?;
        if source.is_empty() {
            return Err(Error::msg("Found empty source file"));
        }

        // submit
        let actor = conf.build_actor();
        let lang_name = conf.service().lang_name();
        actor.submit(&problem, lang_name, &source, cnsl)?;

        Ok(Box::new(SubmitOutcome {
            service: Service::new(conf.global_opt().service_id),
            submitted_at: Local::now(),
            source_bytes: source.len(),
        }))
    }
}

pub type LocalDateTime = DateTime<Local>;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubmitOutcome {
    service: Service,
    submitted_at: LocalDateTime,
    source_bytes: usize,
}

impl fmt::Display for SubmitOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Submitted source to {} ({}, {} Bytes)",
            self.service.id(),
            self.submitted_at
                .to_rfc3339_opts(SecondsFormat::Secs, false),
            self.source_bytes
        )
    }
}

impl Outcome for SubmitOutcome {
    fn is_error(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn run_default() -> anyhow::Result<()> {
        let opt = SubmitOpt {
            problem_id: "c".into(),
            force: true,
        };
        opt.run_default()?;
        Ok(())
    }
}
