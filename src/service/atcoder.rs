use anyhow::{anyhow, Context as _};
use maplit::hashmap;
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, Url};

use crate::model::{Contest, LangNameRef, Problem, ProblemId};
use crate::service::atcoder_page::{
    HasHeader as _, LoginPageBuilder, SettingsPageBuilder, SubmitPageBuilder, TasksPageBuilder,
    TasksPrintPageBuilder, BASE_URL,
};
use crate::service::scrape::{ExtractCsrfToken as _, ExtractLangId as _, HasUrl as _};
use crate::service::session::WithRetry as _;
use crate::service::{Act, ResponseExt as _};
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

impl AtcoderActor<'_> {
    fn submissions_me_url(&self) -> Result<Url> {
        let contest_id = &self.conf.global_opt().contest_id;
        let path = format!("/contests/{}/submissions/me", contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }

    fn validate_login_response(&self, res: &Response) -> Result<()> {
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response code"));
        }
        Ok(())
    }

    fn validate_submit_response(&self, res: &Response) -> Result<()> {
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response code"));
        }
        let loc_url = res
            .location_url(&BASE_URL)
            .context("Could not extract redirection url from response")?;
        if loc_url != self.submissions_me_url()? {
            return Err(Error::msg("Found invalid redirection url"));
        }
        Ok(())
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
        self.validate_login_response(&res)
            .context("Login rejected by service")?;

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

    fn submit(
        &self,
        problem: &Problem,
        lang_name: LangNameRef,
        source: &str,
        cnsl: &mut Console,
    ) -> Result<()> {
        let Self { client, conf } = self;

        let submit_page = SubmitPageBuilder::new(conf).build(client, cnsl)?;
        let csrf_token = submit_page.extract_csrf_token()?;
        let lang_id = submit_page.extract_lang_id(lang_name)?;
        let payload = hashmap!(
            "csrf_token" => csrf_token.as_ref(),
            "data.TaskScreenName" => problem.url_name().as_ref(),
            "data.LanguageId" => lang_id.as_ref(),
            "sourceCode" => source,
        );

        let res = client
            .post(submit_page.url()?)
            .form(&payload)
            .with_retry(client, conf, cnsl)
            .retry_send()?;
        self.validate_submit_response(&res)
            .context("Submission rejected by service")?;

        Ok(())
    }
}
