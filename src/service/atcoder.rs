use maplit::hashmap;
use reqwest::blocking::Client;

use crate::service::atcoder_page::{HasHeader, LoginPageBuilder, SettingsPageBuilder};
use crate::service::request::WithRetry as _;
use crate::service::scrape::{HasUrl as _, Scrape as _};
use crate::service::serve::{LoginOutcome, Serve};
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
        let payload = hashmap!(
            "csrf_token" => login_page.extract_csrf_token()?,
            "username" => user.to_owned(),
            "password" => pass,
        );

        client
            .post(login_page.url())
            .form(&payload)
            .with_retry(client, ctx)
            .retry_send()?;

        let settings_page = SettingsPageBuilder::new()
            .build(client, ctx)?
            .ok_or_else(|| Error::msg("Invalid username or password"))?;
        let current_user = settings_page.current_user()?;
        if current_user != user {
            return Err(Error::msg(format!(
                "Logged in as another user: {}",
                current_user
            )));
        }

        Ok(LoginOutcome {
            service_id: ctx.global_opt.service_id.clone(),
            username: user,
        })
    }
}
