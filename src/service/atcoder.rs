use anyhow::{anyhow, Context as _};
use maplit::hashmap;
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::model::{Contest, Problem, ProblemId};
use crate::service::atcoder_page::{
    HasHeader as _, LoginPageBuilder, SettingsPageBuilder, TasksPageBuilder, TasksPrintPageBuilder,
};
use crate::service::scrape::{ExtractCsrfToken as _, HasUrl as _};
use crate::service::session::WithRetry as _;
use crate::service::Act;
use crate::{Config, Console, Error, Result};

#[derive(Debug)]
pub struct AtcoderActor<'a> {
    client: Client,
    conf: &'a Config,
}

impl<'a> AtcoderActor<'a> {
    pub fn new(client: Client, conf: &'a Config) -> Self {
        AtcoderActor { client, conf }
    }
}

impl Act for AtcoderActor<'_> {
    fn login(&self, user: String, pass: String, cnsl: &mut Console) -> Result<bool> {
        let Self { client, conf } = self;
        let login_page = LoginPageBuilder::new(conf).build(client, cnsl)?;

        // Check if user is already logged in
        if login_page.is_logged_in()? {
            let current_user = login_page.current_user()?;
            if current_user != user {
                return Err(anyhow!("Logged in as another user: {}", current_user));
            }
            return Ok(false);
        }

        // Post form data to log in to service
        let payload = hashmap!(
            "csrf_token" => login_page.extract_csrf_token()?,
            "username" => user.to_owned(),
            "password" => pass,
        );
        let res = client
            .post(login_page.url()?)
            .form(&payload)
            .with_retry(client, conf, cnsl)
            .retry_send()?;
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response"));
        }

        // Check if login succeeded
        let settings_page = SettingsPageBuilder::new(conf).build(client, cnsl)?;
        let current_user = settings_page.current_user()?;
        if current_user != user {
            return Err(anyhow!("Logged in as another user: {}", current_user));
        }

        Ok(true)
    }

    fn fetch(
        &self,
        problem_id: &Option<ProblemId>,
        cnsl: &mut Console,
    ) -> Result<(Contest, Vec<Problem>)> {
        let Self { client, conf } = self;
        let contest_id = &conf.global_opt().contest_id;

        let tasks_page = TasksPageBuilder::new(conf).build(client, cnsl)?;
        let contest_name = tasks_page
            .extract_contest_name()
            .context("Could not extract contest name")?;
        let mut problems: Vec<Problem> = tasks_page
            .extract_problems()?
            .into_iter()
            .filter(|problem| {
                if let Some(problem_id) = problem_id {
                    problem.id() == problem_id
                } else {
                    true
                }
            })
            .collect();
        if problems.is_empty() {
            let err = if let Some(problem_id) = problem_id {
                Err(anyhow!(
                    "Could not find problem \"{}\" in contest {}",
                    problem_id,
                    contest_id
                ))
            } else {
                Err(anyhow!(
                    "Could not find any problems in contest {}",
                    contest_id
                ))
            };
            return err;
        }

        let tasks_print_page = TasksPrintPageBuilder::new(conf).build(client, cnsl)?;
        let mut samples_map = tasks_print_page.extract_samples_map()?;
        for problem in problems.iter_mut() {
            if let Some(samples) = samples_map.remove(&problem.id()) {
                problem.set_samples(samples);
            } else {
                // found problem on TasksPage but not found on TasksPrintPage
                return Err(anyhow!(
                    "Could not extract samples for problem : {}",
                    problem.id()
                ));
            }
        }

        let contest = Contest::new(contest_id.to_owned(), contest_name);
        Ok((contest, problems))
    }
}
