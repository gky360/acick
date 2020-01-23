use once_cell::sync::OnceCell;
use reqwest::{StatusCode, Url};
use scraper::Html;

use crate::service::atcoder_page::BASE_URL;
use crate::service::scrape::{CheckStatus, Scrape};

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

impl CheckStatus for SettingsPage {
    fn is_reject(&self, status: StatusCode) -> bool {
        status.is_redirection() || status.is_client_error() || status == StatusCode::FOUND
    }
}

impl Scrape for SettingsPage {
    fn url(&self) -> Url {
        BASE_URL.join(Self::PATH).unwrap()
    }
}
