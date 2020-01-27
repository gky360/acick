use anyhow::anyhow;
use maplit::hashmap;
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::cmd::LoginOutcome;
use crate::model::{Contest, ProblemId};
use crate::service::atcoder_page::{
    HasHeader as _, LoginPageBuilder, SettingsPageBuilder, TasksPageBuilder, TasksPrintPageBuilder,
};
use crate::service::scrape::{HasUrl as _, Scrape as _};
use crate::service::serve::Serve;
use crate::service::session::WithRetry as _;
use crate::{Context, Error, Result};

#[derive(Debug)]
pub struct AtcoderService<'a, 'b> {
    client: Client,
    ctx: &'a mut Context<'b>,
}

impl<'a, 'b> AtcoderService<'a, 'b> {
    pub fn new(client: Client, ctx: &'a mut Context<'b>) -> Self {
        Self { client, ctx }
    }
}

impl Serve for AtcoderService<'_, '_> {
    fn login(&mut self, user: String, pass: String) -> Result<LoginOutcome> {
        let Self { client, ctx } = self;
        let login_page = LoginPageBuilder::new().build(client, ctx)?;

        // Check if user is already logged in
        if login_page.is_logged_in_as(&user)? {
            return Ok(LoginOutcome {
                service_id: ctx.global_opt.service_id,
                username: user,
                is_already: true,
            });
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
            .with_retry(client, ctx)
            .retry_send()?;
        if res.status() != StatusCode::FOUND {
            return Err(Error::msg("Received invalid response"));
        }

        // Check if login succeeded
        let settings_page = SettingsPageBuilder::new().build(client, ctx)?;
        let current_user = settings_page.current_user()?;
        if current_user != user {
            return Err(anyhow!("Logged in as another user: {}", current_user));
        }

        Ok(LoginOutcome {
            service_id: ctx.global_opt.service_id,
            username: user,
            is_already: false,
        })
    }

    fn fetch(&mut self, problem_id: &Option<ProblemId>) -> Result<Contest> {
        let Self { client, ctx } = self;
        let contest_id = &ctx.global_opt.contest_id;

        let tasks_page = TasksPageBuilder::new(contest_id).build(client, ctx)?;
        let contest = tasks_page.extract_contest()?;

        let tasks_print_page = TasksPrintPageBuilder::new(contest_id).build(client, ctx)?;
        let problems = tasks_print_page
            .extract_problems(problem_id)
            .and_then(|problems| {
                if problems.is_empty() {
                    if let Some(problem_id) = problem_id {
                        Err(anyhow!("Could not find problem \"{}\"", problem_id))
                    } else {
                        Err(anyhow!("Could not find any problems"))
                    }
                } else {
                    Ok(problems)
                }
            })?;
        // TODO: fetch contest name
        let contest = Contest::new(contest_id, "contest.name goes here", problems);
        Ok(contest)
    }
}
