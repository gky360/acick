use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{Fetch as _, HasUrl, Scrape};
use crate::{Config, Context, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageBuilder<'a> {
    conf: &'a Config,
}

impl<'a> SettingsPageBuilder<'a> {
    const PATH: &'static str = "/settings";

    pub fn new(conf: &'a Config) -> Self {
        Self { conf }
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<SettingsPage<'a>> {
        let (status, html) = self.fetch(client, self.conf, ctx)?;
        match status {
            StatusCode::OK => Ok(SettingsPage {
                builder: self,
                content: html,
            }),
            StatusCode::FOUND => Err(Error::msg("Invalid username or password")),
            _ => Err(Error::msg("Received invalid response")),
        }
    }
}

impl HasUrl for SettingsPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        // parsing static path will never fail
        Ok(BASE_URL.join(Self::PATH).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPage<'a> {
    builder: SettingsPageBuilder<'a>,
    content: Html,
}

impl HasUrl for SettingsPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for SettingsPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for SettingsPage<'_> {}
