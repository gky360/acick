use anyhow::Context as _;
use once_cell::sync::OnceCell;
use reqwest::blocking::{Client, Response};
use reqwest::Url;
use retry::{delay, retry, OperationResult};
use scraper::{ElementRef, Html, Selector};

use crate::service::serve::SendPretty as _;
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

pub trait Accept<T> {
    fn is_acceptable(&self, res: &Response) -> bool {
        res.status().is_success()
    }

    fn should_reject(&self, _res: &Response) -> bool {
        false
    }
}

pub trait Scrape: Accept<Response> {
    const HOST: &'static str;
    const PATH: &'static str;

    fn url(&self) -> Url {
        Url::parse(Self::HOST).unwrap().join(Self::PATH).unwrap()
    }

    fn scrape(&self, client: &Client, ctx: &mut Context) -> Result<Html> {
        // TODO: use config
        let durations = delay::Fixed::from_millis(1000).take(4);
        let html = retry(durations, || self.retry_get(client, ctx))
            .map_err(|err| match err {
                retry::Error::Operation { error, .. } => error,
                retry::Error::Internal(msg) => Error::msg(msg),
            })
            .context("Could not get page from service")?;
        Ok(html)
    }

    fn retry_get(&self, client: &Client, ctx: &mut Context) -> OperationResult<Html, Error> {
        let result = client
            .get(self.url())
            .send_pretty(client, ctx)
            .map_err(OperationResult::Retry)
            .and_then(|res| {
                if self.should_reject(&res) {
                    Err(OperationResult::Err(Error::msg(
                        "Received invalid response",
                    )))
                } else if self.is_acceptable(&res) {
                    res.text().map_err(|err| OperationResult::Retry(err.into()))
                } else {
                    Err(OperationResult::Retry(Error::msg("Unacceptable response")))
                }
            })
            .and_then(|text| Ok(Html::parse_document(&text)));
        match result {
            Ok(html) => OperationResult::Ok(html),
            Err(err) => err,
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageBase {
    content_cell: OnceCell<Html>,
}

impl PageBase {
    pub fn new() -> Self {
        Self {
            content_cell: OnceCell::new(),
        }
    }
}

impl AsRef<OnceCell<Html>> for PageBase {
    fn as_ref(&self) -> &OnceCell<Html> {
        &self.content_cell
    }
}

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
