use maplit::hashmap;
use reqwest::blocking::Client;

use crate::service::atcoder_page::{LoginPageBuilder, SettingsPage};
use crate::service::request::WithRetry as _;
use crate::service::scrape::{HasUrl as _, Scrape as _};
use crate::service::serve::{LoginOutcome, Serve};
use crate::{Context, Result};

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

        let _settings_page = SettingsPage::new();

        let outcome = LoginOutcome {
            service_id: ctx.global_opt.service_id.clone(),
            username: user,
        };
        Ok(outcome)
    }
}
