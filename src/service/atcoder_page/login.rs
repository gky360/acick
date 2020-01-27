use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html};

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{CheckStatus, Fetch as _, HasUrl, Scrape};
use crate::{Context, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPageBuilder {}

impl LoginPageBuilder {
    const PATH: &'static str = "/login";

    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<LoginPage> {
        self.fetch(client, ctx)?
            .ok_or_else(|| Error::msg("Received invalid page"))
            .map(|html| LoginPage {
                builder: self,
                content: html,
            })
    }
}

impl CheckStatus for LoginPageBuilder {}

impl HasUrl for LoginPageBuilder {
    fn url(&self) -> Result<Url> {
        // parsing static path will never fail
        Ok(BASE_URL.join(Self::PATH).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPage {
    builder: LoginPageBuilder,
    content: Html,
}

impl HasUrl for LoginPage {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for LoginPage {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for LoginPage {}
