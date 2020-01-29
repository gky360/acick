use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html};

use crate::model::{Problem, ProblemId};
use crate::service::atcoder_page::{FetchMaybeNotFound, HasHeader, BASE_URL};
use crate::service::scrape::{select, ElementRefExt as _, HasUrl, Scrape};
use crate::{Config, Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPageBuilder<'a> {
    conf: &'a Config,
}

impl<'a> TasksPageBuilder<'a> {
    pub fn new(conf: &'a Config) -> Self {
        Self { conf }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<TasksPage<'a>> {
        self.fetch_maybe_not_found(client, self.conf, cnsl)
            .map(|html| TasksPage {
                builder: self,
                content: html,
            })
    }
}

impl HasUrl for TasksPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let contest_id = &self.conf.global_opt().contest_id;
        let path = format!("/contests/{}/tasks", contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl FetchMaybeNotFound for TasksPageBuilder<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPage<'a> {
    builder: TasksPageBuilder<'a>,
    content: Html,
}

impl TasksPage<'_> {
    pub fn extract_problems(&self) -> Result<Vec<Problem>> {
        self.select_problem_rows()
            .map(|elem| elem.extract_problem())
            .collect()
    }

    fn select_problem_rows(&self) -> impl Iterator<Item = ProblemRowElem> {
        self.content
            .select(select!("#main-container .panel table tbody tr"))
            .map(ProblemRowElem)
    }
}

impl HasUrl for TasksPage<'_> {
    fn url(&self) -> Result<Url> {
        self.builder.url()
    }
}

impl Scrape for TasksPage<'_> {
    fn elem(&self) -> ElementRef {
        self.content.root_element()
    }
}

impl HasHeader for TasksPage<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProblemRowElem<'a>(ElementRef<'a>);

impl ProblemRowElem<'_> {
    // TODO: extract time and memory limits
    fn extract_problem(&self) -> Result<Problem> {
        let mut iter = self.0.select(select!("td"));
        let id = iter
            .next()
            .map(|td| ProblemId::from(td.inner_text().trim()))
            .context("Could not find task id")?;
        let name = iter
            .next()
            .map(|td| td.inner_text().trim().to_owned())
            .context("Could not find task name")?;
        let url = self
            .find_first(select!("a"))
            .context("Could not find link to a task")?
            .value()
            .attr("href")
            .and_then(|href| BASE_URL.join(href).ok())
            .context("Could not parse task url")?;
        Ok(Problem::new(id, name, url, Vec::new()))
    }
}

impl Scrape for ProblemRowElem<'_> {
    fn elem(&self) -> ElementRef {
        self.0
    }
}
