use lazy_static::lazy_static;
use reqwest::Url;
use scraper::ElementRef;

use crate::service::scrape::{select, ElementRefExt as _, Scrape};
use crate::{Error, Result};

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
