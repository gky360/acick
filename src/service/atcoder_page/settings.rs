use reqwest::blocking::Client;
use reqwest::Url;
use scraper::Html;

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{CheckStatus, Fetch as _, HasUrl};
use crate::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageBuilder {}

impl SettingsPageBuilder {
    const PATH: &'static str = "/settings";

    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<Option<SettingsPage>> {
        let maybe_page = self.fetch(client, ctx)?.map(|html| SettingsPage {
            builder: self,
            content: html,
        });
        Ok(maybe_page)
    }
}

impl CheckStatus for SettingsPageBuilder {}

impl HasUrl for SettingsPageBuilder {
    fn url(&self) -> Result<Url> {
        // parsing static path will never fail
        Ok(BASE_URL.join(Self::PATH).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPage {
    builder: SettingsPageBuilder,
    content: Html,
}

impl HasUrl for SettingsPage {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl AsRef<Html> for SettingsPage {
    fn as_ref(&self) -> &Html {
        &self.content
    }
}

impl HasHeader for SettingsPage {}
