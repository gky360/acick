use reqwest::blocking::Client;
use reqwest::Url;
use scraper::Html;

use crate::service::atcoder_page::BASE_URL;
use crate::service::scrape::{CheckStatus, Fetch as _, HasUrl};
use crate::{Context, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPage {
    builder: LoginPageBuilder,
    content: Html,
}

impl HasUrl for LoginPage {
    fn url(&self) -> Url {
        self.builder.url()
    }
}

impl AsRef<Html> for LoginPage {
    fn as_ref(&self) -> &Html {
        &self.content
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPageBuilder {}

impl LoginPageBuilder {
    const PATH: &'static str = "/login";

    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<LoginPage> {
        let html = self
            .fetch(client, ctx)?
            .ok_or_else(|| Error::msg("Received invalid page"))?;
        Ok(LoginPage {
            builder: self,
            content: html,
        })
    }
}

impl CheckStatus for LoginPageBuilder {}

impl HasUrl for LoginPageBuilder {
    fn url(&self) -> Url {
        BASE_URL.join(Self::PATH).unwrap()
    }
}
