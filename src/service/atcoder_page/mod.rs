use anyhow::Context as _;
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::config::SessionConfig;
use crate::macros::select;
use crate::service::scrape::{ElementRefExt as _, Fetch, Scrape};
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

    fn current_user(&self) -> Result<String> {
        if !self.is_logged_in()? {
            return Err(Error::msg("Not logged in"));
        }
        self.select_header()?
            .select(select!("a.dropdown-toggle"))
            .nth(1)
            .ok_or_else(|| Error::msg("Could not find element"))
            .map(|elem| elem.inner_text().trim().to_owned())
    }

    fn is_logged_in_as(&self, user: &str) -> Result<bool> {
        Ok(self.is_logged_in()? && self.current_user()? == user)
    }

    fn extract_contest_name(&self) -> Option<String> {
        self.find_first(select!(".contest-title"))
            .map(|elem| elem.inner_text().trim().to_owned())
    }
}

pub trait FetchRestricted: Fetch {
    fn fetch_restricted(
        &self,
        client: &Client,
        session: &SessionConfig,
        cnsl: &mut Console,
    ) -> Result<Html> {
        let (status, html) = self.fetch(client, session, cnsl)?;
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
