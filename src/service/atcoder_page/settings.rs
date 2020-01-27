use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{Fetch as _, HasUrl, Scrape};
use crate::{Context, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPageBuilder {}

impl SettingsPageBuilder {
    const PATH: &'static str = "/settings";

    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<SettingsPage> {
        let (status, html) = self.fetch(client, ctx)?;
        if status == StatusCode::OK {
            Ok(SettingsPage {
                builder: self,
                content: html,
            })
        } else {
            Err(Error::msg("Invalid username or password"))
        }
    }
}

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

impl Scrape for SettingsPage {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for SettingsPage {}
