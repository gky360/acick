use lazy_static::lazy_static;
use reqwest::Url;
use scraper::ElementRef;

use crate::service::scrape::{select, Scrape};
use crate::{Error, Result};

mod login;
mod settings;

pub use login::{LoginPage, LoginPageBuilder};
pub use settings::{SettingsPage, SettingsPageBuilder};

lazy_static! {
    pub static ref BASE_URL: Url = Url::parse("https://atcoder.jp").unwrap();
}

pub trait HasHeader: Scrape {
    fn select_header(&self) -> Result<ElementRef> {
        self.find_first(select!("nav"))
    }

    fn current_user(&self) -> Result<String> {
        self.select_header()?
            .select(select!("a.dropdown-toggle"))
            .nth(1)
            .ok_or_else(|| Error::msg("Could not find element"))
            .map(|elem| {
                elem.text()
                    .collect::<Vec<&str>>()
                    .join("")
                    .trim()
                    .to_owned()
            })
    }

    fn is_logged_in_as(&self, user: &str) -> Result<bool> {
        Ok(self.current_user()? == user)
    }
}
