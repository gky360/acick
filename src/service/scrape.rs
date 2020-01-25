use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};

use crate::service::session::WithRetry as _;
use crate::{Context, Error, Result};

#[macro_export]
macro_rules! select {
    ($selectors:literal) => {{
        static SELECTOR: ::once_cell::sync::Lazy<::scraper::selector::Selector> =
            ::once_cell::sync::Lazy::new(|| {
                ::scraper::selector::Selector::parse($selectors).unwrap()
            });
        &SELECTOR
    }};
    ($selectors:literal,) => {
        selector!($selectors)
    };
}
pub use select;

pub trait HasUrl {
    fn url(&self) -> Url;
}

pub trait CheckStatus {
    fn is_accept(&self, status: StatusCode) -> bool {
        status.is_success()
    }

    fn is_reject(&self, status: StatusCode) -> bool {
        status.is_redirection() || status.is_client_error()
    }
}

pub trait Fetch: HasUrl + CheckStatus {
    fn fetch(&self, client: &Client, ctx: &mut Context) -> Result<Option<Html>> {
        let maybe_html = client
            .get(self.url())
            .with_retry(client, ctx)
            .accept(|status| self.is_accept(status))
            .reject(|status| self.is_reject(status))
            .retry_send()?
            .map(|res| res.text())
            .transpose()?
            .map(|text| Html::parse_document(&text));
        Ok(maybe_html)
    }
}

impl<T: HasUrl + CheckStatus> Fetch for T {}

pub trait Scrape: AsRef<Html> {
    fn find_first(&self, selector: &Selector) -> Result<ElementRef> {
        self.as_ref()
            .select(selector)
            .next()
            .context("Could not find element")
    }

    fn extract_csrf_token(&self) -> Result<String> {
        let token = self
            .find_first(select!("[name=\"csrf_token\"]"))?
            .value()
            .attr("value")
            .context("Could not find csrf_token value attr")?
            .to_owned();
        if token.is_empty() {
            Err(Error::msg("Found empty csrf token"))
        } else {
            Ok(token)
        }
    }
}

impl<T: AsRef<Html>> Scrape for T {}
