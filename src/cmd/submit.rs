use std::fmt;
use std::io::Write as _;

use anyhow::Context as _;
use chrono::{offset::Local, DateTime, SecondsFormat};
use serde::Serialize;
use structopt::StructOpt;

use crate::cmd::Outcome;
use crate::model::{ProblemId, Service};
use crate::{Config, Console, Error, Result};

#[derive(StructOpt, Debug, Clone, PartialEq, Eq, Hash)]
#[structopt(rename_all = "kebab")]
pub struct SubmitOpt {
    /// Id of the problem to be submitted
    #[structopt(name = "problem")]
    problem_id: ProblemId,
    /// Submits without prompting for confirmation
    #[structopt(long, short)]
    yes: bool,
    /// Opens the submission status in browser
    #[structopt(name = "open", long, short)]
    need_open: bool,
}

impl SubmitOpt {
    pub fn run(&self, conf: &Config, cnsl: &mut Console) -> Result<SubmitOutcome> {
        // confirm
        let message = format!(
            "submit problem {} to {}?",
            &self.problem_id, &conf.contest_id
        );
        if !self.yes && !cnsl.confirm(&message, false)? {
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
        actor.submit(&conf.contest_id, &problem, lang_name, &source, cnsl)?;

        // open submissions in browser if needed
        if self.need_open {
            actor
                .open_submissions_url(&conf.contest_id, cnsl)
                // coerce error
                .unwrap_or_else(|err| writeln!(cnsl, "{:?}", err).unwrap_or(()));
        }

        Ok(SubmitOutcome {
            service: Service::new(conf.service_id),
            submitted_at: Local::now(),
            source_bytes: source.len(),
        })
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
    use tempfile::tempdir;

    use super::*;
    use crate::cmd::tests::run_with;

    #[test]
    #[ignore]
    fn run_default() -> anyhow::Result<()> {
        let test_dir = tempdir()?;

        let login_opt = crate::cmd::LoginOpt {};
        run_with(&test_dir, |conf, cnsl| login_opt.run(conf, cnsl))?;

        let fetch_opt = crate::cmd::FetchOpt::default_test();
        run_with(&test_dir, |conf, cnsl| fetch_opt.run(conf, cnsl))?;

        let opt = SubmitOpt {
            problem_id: "c".into(),
            yes: true,
            need_open: false,
        };
        run_with(&test_dir, |conf, cnsl| opt.run(conf, cnsl))?;
        Ok(())
    }
}
