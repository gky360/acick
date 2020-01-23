use anyhow::Context as _;
use once_cell::sync::OnceCell;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};

use crate::service::request::WithRetry as _;
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
use select;

pub trait CheckStatus {
    fn is_accept(&self, status: StatusCode) -> bool {
        status.is_success()
    }

    fn is_reject(&self, status: StatusCode) -> bool {
        status.is_redirection() || status.is_client_error()
    }
}

pub trait Scrape: CheckStatus {
    fn url(&self) -> Url;

    fn scrape(&self, client: &Client, ctx: &mut Context) -> Result<Html> {
        let res = client
            .get(self.url())
            .with_retry(client, ctx)
            .accept(|status| self.is_accept(status))
            .reject(|status| self.is_reject(status))
            .retry_send()?
            .unwrap(); // TODO: fix
        let html = Html::parse_document(&res.text()?);
        Ok(html)
    }
}

pub trait ScrapeOnce: Scrape + AsRef<OnceCell<Html>> {
    fn content(&self, client: &Client, ctx: &mut Context) -> Result<&Html> {
        let html = self
            .as_ref()
            .get_or_try_init(|| self.scrape(client, ctx))
            .context("Could not get page content from service")?;
        Ok(html)
    }
}

impl<T: Scrape + AsRef<OnceCell<Html>>> ScrapeOnce for T {}

pub trait Extract {
    fn find_first(&self, selector: &Selector) -> Result<ElementRef>;
    fn extract_csrf_token(&self) -> Result<String>;
}

impl Extract for Html {
    fn find_first(&self, selector: &Selector) -> Result<ElementRef> {
        self.select(selector)
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
