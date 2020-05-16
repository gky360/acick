use acick_util::select;
use anyhow::Context as _;
use humantime::parse_duration;
use reqwest::blocking::Client;
use reqwest::Url;
use scraper::{ElementRef, Html};

use crate::config::SessionConfig;
use crate::model::{Compare, ContestId, Problem, ProblemId};
use crate::page::{GetHtmlRestricted, HasHeader, BASE_URL};
use crate::service::scrape::{GetHtml, Scrape};
use crate::{Console, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPageBuilder<'a> {
    contest_id: &'a ContestId,
    session: &'a SessionConfig,
}

impl<'a> TasksPageBuilder<'a> {
    pub fn new(contest_id: &'a ContestId, session: &'a SessionConfig) -> Self {
        Self {
            contest_id,
            session,
        }
    }

    pub fn build(self, client: &Client, cnsl: &mut Console) -> Result<TasksPage<'a>> {
        self.get_html_restricted(client, self.session, cnsl)
            .map(|html| TasksPage {
                builder: self,
                content: html,
            })
    }
}

impl GetHtml for TasksPageBuilder<'_> {
    fn url(&self) -> Result<Url> {
        let path = format!("/contests/{}/tasks", self.contest_id);
        BASE_URL
            .join(&path)
            .context(format!("Could not parse url path: {}", path))
    }
}

impl GetHtmlRestricted for TasksPageBuilder<'_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TasksPage<'a> {
    builder: TasksPageBuilder<'a>,
    content: Html,
}

impl TasksPage<'_> {
    pub fn extract_problems(&self, cnsl: &mut Console) -> Result<Vec<Problem>> {
        self.select_problem_rows()
            .map(|elem| elem.extract_problem(cnsl))
            .collect()
    }

    fn select_problem_rows(&self) -> impl Iterator<Item = ProblemRowElem> {
        self.content
            .select(select!("#main-container .panel table tbody tr"))
            .map(ProblemRowElem)
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
    fn extract_problem(&self, cnsl: &mut Console) -> Result<Problem> {
        let mut iter = self.0.select(select!("td"));
        let id = iter
            .next()
            .map(|td| ProblemId::from(td.inner_text().trim()))
            .context("Could not find task id")?;
        let name = iter
            .next()
            .map(|td| td.inner_text().trim().to_owned())
            .context("Could not find task name")?;
        let time_limit = iter
            .next()
            .and_then(|td| parse_duration(td.inner_text().trim()).ok());
        if time_limit.is_none() {
            cnsl.warn("Could not parse time limit")?;
        }
        let memory_limit = iter
            .next()
            .and_then(|td| td.inner_text().trim().parse().ok());
        if memory_limit.is_none() {
            cnsl.warn("Could not parse memory limit")?;
        }
        let task_url = self
            .find_first(select!("a"))
            .context("Could not find link to a task")?
            .value()
            .attr("href")
            .and_then(|href| BASE_URL.join(href).ok())
            .context("Could not parse task url")?;
        let url_name = task_url
            .path_segments()
            .and_then(|segs| segs.last())
            .context("Could not parse url_name")?;
        Ok(Problem::new(
            id,
            name,
            url_name,
            time_limit,
            memory_limit,
            Compare::Default, // TODO: support float
            Vec::new(),
        ))
    }
}

impl Scrape for ProblemRowElem<'_> {
    fn elem(&self) -> ElementRef {
        self.0
    }
}
