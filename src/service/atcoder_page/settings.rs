use once_cell::sync::OnceCell;
use reqwest::blocking::Response;
use reqwest::{StatusCode, Url};
use scraper::Html;

use crate::service::atcoder_page::BASE_URL;
use crate::service::scrape::{Accept, Scrape};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPage {
    content_cell: OnceCell<Html>,
}

impl SettingsPage {
    const PATH: &'static str = "/settings";

    pub fn new() -> Self {
        Self {
            content_cell: OnceCell::new(),
        }
    }
}

impl AsRef<OnceCell<Html>> for SettingsPage {
    fn as_ref(&self) -> &OnceCell<Html> {
        &self.content_cell
    }
}

impl Accept<Response> for SettingsPage {
    fn should_reject(&self, res: &Response) -> Result<()> {
        if res.status() == StatusCode::FOUND {
            Ok(())
        } else {
            Err(Error::msg("User not logged in"))
        }
    }
}

impl Scrape for SettingsPage {
    fn url(&self) -> Url {
        BASE_URL.join(Self::PATH).unwrap()
    }
}
