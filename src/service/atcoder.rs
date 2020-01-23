use maplit::hashmap;
use reqwest::blocking::Client;
use reqwest::StatusCode;

use crate::service::atcoder_page::{LoginPage, SettingsPage};
use crate::service::scrape::{Extract as _, Scrape as _, ScrapeOnce as _};
use crate::service::serve::{LoginOutcome, SendPretty as _, Serve};
use crate::utils::WithRetry as _;
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
        let login_page = LoginPage::new();
        let payload = hashmap!(
            "csrf_token" => login_page.content(&self.client, self.ctx)?.extract_csrf_token()?,
            "username" => user.to_owned(),
            "password" => pass,
        );
        self.client
            .post(login_page.url())
            .form(&payload)
            .with_retry(&self.client, self.ctx)
            .retry_send()?;

        let _settings_page = SettingsPage::new();

        let outcome = LoginOutcome {
            service_id: self.ctx.global_opt.service_id.clone(),
            username: user,
        };
        Ok(outcome)
    }
}
