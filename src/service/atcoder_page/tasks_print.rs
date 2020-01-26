use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::Html;

use crate::service::atcoder_page::{HasHeader, BASE_URL};
use crate::service::scrape::{CheckStatus, Fetch as _, HasUrl};
use crate::{Context, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPrintPageBuilder<'a> {
    contest_id: &'a str,
}

impl<'a> TasksPrintPageBuilder<'a> {
    pub fn new(contest_id: &'a str) -> Self {
        Self { contest_id }
    }

    pub fn build(self, client: &Client, ctx: &mut Context) -> Result<TasksPrintPage<'a>> {
        self.fetch(client, ctx)?
            .ok_or_else(|| Error::msg("Received invalid page"))
            .map(|html| TasksPrintPage {
                builder: self,
                content: html,
            })
    }
}

impl CheckStatus for TasksPrintPageBuilder<'_> {}

impl HasUrl for TasksPrintPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let path = format!("/contests/{}/tasks_print", self.contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPrintPage<'a> {
    builder: TasksPrintPageBuilder<'a>,
    content: Html,
}

impl HasUrl for TasksPrintPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl AsRef<Html> for TasksPrintPage<'_> {
    fn as_ref(&self) -> &Html {
        &self.content
    }
}

impl HasHeader for TasksPrintPage<'_> {}
