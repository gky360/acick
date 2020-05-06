use std::str::FromStr;
use std::time::Duration;

use anyhow::Context as _;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use scraper::{ElementRef, Html, Selector};

use crate::abs_path::AbsPathBuf;
use crate::model::{LangId, LangNameRef};
use crate::select;
use crate::service::session::WithRetry as _;
use crate::{Console, Error, Result};

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

pub trait Fetch {
    fn url(&self) -> Result<Url>;

    fn fetch(
        &self,
        client: &Client,
        cookies_path: &AbsPathBuf,
        retry_limit: usize,
        retry_interval: Duration,
        cnsl: &mut Console,
    ) -> Result<(StatusCode, Html)> {
        let res = client
            .get(self.url()?)
            .with_retry(client, cookies_path, retry_limit, retry_interval, cnsl)
            .retry_send()?;
        let status = res.status();
        let html = res.text().map(|text| Html::parse_document(&text))?;
        Ok((status, html))
    }
}

pub trait Scrape {
    fn elem(&self) -> ElementRef;

    fn find_first(&self, selector: &Selector) -> Option<ElementRef> {
        self.elem().select(selector).next()
    }

    fn inner_text(&self) -> String {
        self.elem().text().fold(String::new(), |mut ret, s| {
            ret.push_str(s);
            ret
        })
    }
}

impl Scrape for ElementRef<'_> {
    fn elem(&self) -> ElementRef {
        *self
    }
}

pub trait ExtractCsrfToken: Scrape {
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

pub trait ExtractLangId {
    fn extract_lang_id(&self, lang_name: LangNameRef) -> Option<LangId>;
}
