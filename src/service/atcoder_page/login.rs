use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{Fetch as _, HasUrl, Scrape};
use crate::{Config, Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPageBuilder<'a> {
    conf: &'a Config,
}

impl<'a> LoginPageBuilder<'a> {
    const PATH: &'static str = "/login";

    pub fn new(conf: &'a Config) -> Self {
        Self { conf }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<LoginPage<'a>> {
        self.fetch_if(|s| s == StatusCode::OK, client, self.conf, cnsl)
            .map(|html| LoginPage {
                builder: self,
                content: html,
            })
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
