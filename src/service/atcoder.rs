use anyhow::anyhow;
use maplit::hashmap;
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::cmd::LoginOutcome;
use crate::model::{Problem, ProblemId};
use crate::service::atcoder_page::{
    HasHeader as _, LoginPageBuilder, SettingsPageBuilder, TasksPrintPageBuilder,
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
        client
            .post(login_page.url()?)
            .form(&payload)
            .with_retry(client, ctx)
            .accept(|status| status == StatusCode::FOUND)
            .retry_send()?
            .ok_or_else(|| Error::msg("Received invalid response"))?;

        // Check if login succeeded
        let settings_page = SettingsPageBuilder::new()
            .build(client, ctx)?
            .ok_or_else(|| Error::msg("Invalid username or password"))?;
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

    fn fetch(&mut self, _problem_id: &Option<ProblemId>) -> Result<Vec<Problem>> {
        let Self { client, ctx } = self;
        let contest_id = &ctx.global_opt.contest_id;
        let _tasks_print_page = TasksPrintPageBuilder::new(contest_id).build(client, ctx)?;
        Ok(vec![])
    }
}
