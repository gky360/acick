use once_cell::sync::OnceCell;
use reqwest::Url;
use scraper::Html;

use crate::service::atcoder_page::BASE_URL;
use crate::service::scrape::{CheckStatus, Scrape};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPage {
    content_cell: OnceCell<Html>,
}

impl LoginPage {
    const PATH: &'static str = "/login";

    pub fn new() -> Self {
        Self {
            content_cell: OnceCell::new(),
        }
    }
}

impl AsRef<OnceCell<Html>> for LoginPage {
    fn as_ref(&self) -> &OnceCell<Html> {
        &self.content_cell
    }
}

impl CheckStatus for LoginPage {}

impl Scrape for LoginPage {
    fn url(&self) -> Url {
        BASE_URL.join(Self::PATH).unwrap()
    }
}
