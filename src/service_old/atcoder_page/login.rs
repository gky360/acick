use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::config::SessionConfig;
use crate::service::scrape::{ExtractCsrfToken, Fetch as _, HasUrl, Scrape};
use crate::service_old::atcoder_page::{HasHeader, BASE_URL};
use crate::{Console, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPageBuilder<'a> {
    session: &'a SessionConfig,
}

impl<'a> LoginPageBuilder<'a> {
    const PATH: &'static str = "/login";

    pub fn new(session: &'a SessionConfig) -> Self {
        Self { session }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<LoginPage<'a>> {
        let (status, html) = self.fetch(
            client,
            self.session.retry_limit(),
            self.session.retry_interval(),
            cnsl,
        )?;
        match status {
            StatusCode::OK => Ok(LoginPage {
                builder: self,
                content: html,
            }),
            _ => Err(Error::msg("Received invalid response")),
        }
    }
}

impl HasUrl for LoginPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        // parsing static path will never fail
        Ok(BASE_URL.join(Self::PATH).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPage<'a> {
    builder: LoginPageBuilder<'a>,
    content: Html,
}

impl HasUrl for LoginPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for LoginPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for LoginPage<'_> {}

impl ExtractCsrfToken for LoginPage<'_> {}