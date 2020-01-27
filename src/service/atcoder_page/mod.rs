use anyhow::{anyhow, Context as _};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html};

use crate::service::scrape::{select, ElementRefExt as _, Fetch, Scrape};
use crate::{Context, Error, Result};

mod login;
mod settings;
mod tasks_print;

pub use login::{LoginPage, LoginPageBuilder};
pub use settings::{SettingsPage, SettingsPageBuilder};
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
}

pub trait FetchMaybeNotFound: Fetch {
    fn fetch_maybe_not_found(&self, client: &Client, ctx: &mut Context) -> Result<Html> {
        let (status, html) = self.fetch(client, ctx)?;
        match status {
            StatusCode::OK => Ok(html),
            StatusCode::NOT_FOUND if NotFoundPage(&html).is_not_found() => Err(anyhow!(
                "Could not find contest : {} .
Check if the contest id is correct.",
                ctx.global_opt.contest_id
            )),
            StatusCode::NOT_FOUND if NotFoundPage(&html).is_permission_denied() => Err(anyhow!(
                "Found not participated or not started contest : {} .
Participate in the contest and wait until the contest starts.",
                ctx.global_opt.contest_id
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
