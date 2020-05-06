use acick_util::select;
use anyhow::Context as _;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::config::SessionConfig;
use crate::model::{LangId, LangNameRef};
use crate::service::scrape::{GetHtml, Scrape};
use crate::{Console, Error, Result};

mod login;
mod settings;
mod submit;
mod tasks;
mod tasks_print;

pub use login::{LoginPage, LoginPageBuilder};
pub use settings::{SettingsPage, SettingsPageBuilder};
pub use submit::{SubmitPage, SubmitPageBuilder};
pub use tasks::{TasksPage, TasksPageBuilder};
pub use tasks_print::{TasksPrintPage, TasksPrintPageBuilder};

lazy_static! {
    pub static ref BASE_URL: Url = Url::parse("https://atcoder.jp").unwrap();
}

pub trait ExtractCsrfToken: Scrape {
    fn extract_csrf_token(&self) -> Result<&str> {
        let token = self
            .find_first(select!("[name=\"csrf_token\"]"))
            .context("Could not extract csrf token")?
            .value()
            .attr("value")
            .context("Could not find csrf_token value attr")?;
        if token.is_empty() {
            Err(Error::msg("Found empty csrf token"))
        } else {
            Ok(token)
        }
    }
}

pub trait ExtractLangId {
    fn extract_lang_id(&self, lang_name: LangNameRef) -> Option<LangId>;
}

pub trait HasHeader: Scrape {
    fn select_header(&self) -> Result<ElementRef> {
        self.find_first(select!("nav"))
            .context("Could not find header")
    }

    fn is_logged_in(&self) -> Result<bool> {
        let ret = self
            .select_header()?
            .select(select!("a.dropdown-toggle .glyphicon-cog"))
            .next()
            .is_some();
        Ok(ret)
    }

    fn current_user(&self) -> Result<Option<String>> {
        if !self.is_logged_in()? {
            return Ok(None);
        }
        self.select_header()?
            .select(select!("a.dropdown-toggle"))
            .nth(1)
            .ok_or_else(|| Error::msg("Could not find element"))
            .map(|elem| Some(elem.inner_text().trim().to_owned()))
    }

    fn is_logged_in_as(&self, user: &str) -> Result<bool> {
        match self.current_user()? {
            None => Ok(false),
            Some(current_user) => Ok(current_user == user),
        }
    }

    fn extract_contest_name(&self) -> Option<String> {
        self.find_first(select!(".contest-title"))
            .map(|elem| elem.inner_text().trim().to_owned())
    }
}

pub trait GetHtmlRestricted: GetHtml {
    fn get_html_restricted(
        &self,
        client: &Client,
        session: &SessionConfig,
        cnsl: &mut Console,
    ) -> Result<Html> {
        let (status, html) = self.get_html(
            client,
            session.cookies_path(),
            session.retry_limit(),
            session.retry_interval(),
            cnsl,
        )?;
        match status {
            StatusCode::OK => Ok(html),
            StatusCode::FOUND => Err(Error::msg("User not logged in")),
            StatusCode::NOT_FOUND if NotFoundPage(&html).is_not_found() => Err(Error::msg(
                "Could not find contest. Check if the contest id is correct.",
            )),
            StatusCode::NOT_FOUND if NotFoundPage(&html).is_permission_denied() => Err(Error::msg(
                "Found not participated or not started contest. Participate in the contest and wait until the contest starts.",
            )),
            _ => Err(Error::msg("Received invalid response")),
        }
    }
}

struct NotFoundPage<'a>(&'a Html);

impl NotFoundPage<'_> {
    fn select_alert(&self) -> Option<ElementRef> {
        self.find_first(select!(".alert-danger"))
    }

    fn alert_contains(&self, pat: &str) -> bool {
        self.select_alert()
            .map(|elem| elem.inner_text().contains(pat))
            .unwrap_or(false)
    }

    fn is_permission_denied(&self) -> bool {
        self.alert_contains("Permission denied.")
    }

    fn is_not_found(&self) -> bool {
        self.alert_contains("Contest not found.")
    }
}

impl Scrape for NotFoundPage<'_> {
    fn elem(&self) -> ElementRef {
        self.0.root_element()
    }
}
