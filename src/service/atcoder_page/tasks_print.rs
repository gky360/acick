use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{html::Select, ElementRef, Html};

use crate::model::Problem;
use crate::service::atcoder_page::BASE_URL;
use crate::service::scrape::{select, CheckStatus, ElementRefExt as _, Fetch as _, HasUrl};
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

impl TasksPrintPage<'_> {
    pub fn extract_problems(&self) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();
        for elem in self.select_problems() {
            let pe = ProblemElem(elem);
            let (id, _) = pe.select_id_name()?;
            problems.push(Problem::new(id));
        }
        Ok(problems)
    }

    fn select_problems(&self) -> Select<'_, '_> {
        self.content.select(select!(
            "#main-container > .row > .col-sm-12:not(.next-page)"
        ))
    }
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

struct ProblemElem<'a>(ElementRef<'a>);

impl ProblemElem<'_> {
    fn select_id_name(&self) -> Result<(String, String)> {
        let title = self
            .0
            .select(select!(".h2"))
            .next()
            .ok_or_else(|| Error::msg("Could not find problem title"))?
            .inner_text();
        let mut id_name = title.splitn(2, '-');
        let id = id_name
            .next()
            .ok_or_else(|| Error::msg("Could not find problem id"))?
            .trim();
        let name = id_name
            .next()
            .ok_or_else(|| Error::msg("Could not find problem name"))?
            .trim();
        Ok((id.to_owned(), name.to_owned()))
    }
}
