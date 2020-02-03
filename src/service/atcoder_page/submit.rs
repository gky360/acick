use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html};

use crate::service::atcoder_page::{FetchRestricted, HasHeader, BASE_URL};
use crate::service::scrape::{HasUrl, Scrape};
use crate::{Config, Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitPageBuilder<'a> {
    conf: &'a Config,
}

impl<'a> SubmitPageBuilder<'a> {
    pub fn new(conf: &'a Config) -> Self {
        Self { conf }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<SubmitPage<'a>> {
        self.fetch_restricted(client, self.conf, cnsl)
            .map(|html| SubmitPage {
                builder: self,
                content: html,
            })
    }
}

impl HasUrl for SubmitPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let contest_id = &self.conf.global_opt().contest_id;
        let path = format!("/contests/{}/submit", contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl FetchRestricted for SubmitPageBuilder<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitPage<'a> {
    builder: SubmitPageBuilder<'a>,
    content: Html,
}

impl HasUrl for SubmitPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for SubmitPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for SubmitPage<'_> {}
