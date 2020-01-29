use std::str::FromStr;

use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};

use crate::service::session::WithRetry as _;
use crate::{Config, Console, Error, Result};

#[macro_export]
macro_rules! regex {
    ($expr:expr) => {{
        static REGEX: ::once_cell::sync::Lazy<::regex::Regex> =
            ::once_cell::sync::Lazy::new(|| ::regex::Regex::new($expr).unwrap());
        &REGEX
    }};
    ($expr:expr,) => {
        lazy_regex!($expr)
    };
}
pub use regex;

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
    fn url(&self) -> Result<Url>;
}

pub trait Fetch: HasUrl {
    fn fetch(
        &self,
        client: &Client,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<(StatusCode, Html)> {
        let res = client
            .get(self.url()?)
            .with_retry(client, conf, cnsl)
            .retry_send()?;
        let status = res.status();
        let html = res.text().map(|text| Html::parse_document(&text))?;
        Ok((status, html))
    }

    fn fetch_if(
        &self,
        check: impl FnOnce(StatusCode) -> bool,
        client: &Client,
        conf: &Config,
        cnsl: &mut Console,
    ) -> Result<Html> {
        let (status, html) = self.fetch(client, conf, cnsl)?;
        if check(status) {
            Ok(html)
        } else {
            Err(Error::msg("Received invalid response"))
        }
    }
}

impl<T: HasUrl> Fetch for T {}

pub trait Scrape {
    fn elem(&self) -> ElementRef;

    fn find_first(&self, selector: &Selector) -> Option<ElementRef> {
        self.elem().select(selector).next()
    }

    fn extract_csrf_token(&self) -> Result<String> {
        let token = self
            .find_first(select!("[name=\"csrf_token\"]"))
            .context("Could not extract csrf token")?
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

pub trait ElementRefExt {
    fn inner_text(&self) -> String;
}

impl ElementRefExt for ElementRef<'_> {
    fn inner_text(&self) -> String {
        self.text().fold("".to_owned(), |mut ret, s| {
            ret.push_str(s);
            ret
        })
    }
}

pub fn parse_zenkaku_digits<T: FromStr>(s: &str) -> std::result::Result<T, T::Err> {
    s.parse().or_else(|err| {
        if s.chars().all(|c| '０' <= c && c <= '９') {
            s.chars()
                .map(|c| char::from((u32::from(c) - u32::from('０') + u32::from('0')) as u8))
                .collect::<String>()
                .parse()
        } else {
            Err(err)
        }
    })
}
