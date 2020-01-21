use std::fmt;

use anyhow::Context as _;
use reqwest::blocking::{Client, Response};
use reqwest::Url;
use retry::{delay, retry, OperationResult};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::model::ServiceKind;
use crate::{Context, Error, Result};

mod atcoder;

pub use atcoder::AtcoderService;

pub static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

trait Accept<T> {
    fn is_acceptable(&self, data: &T) -> bool;
}

trait Scrape: Accept<Response> {
    const HOST: &'static str;
    const PATH: &'static str;

    fn url(&self) -> Url {
        Url::parse(Self::HOST).unwrap().join(Self::PATH).unwrap()
    }

    fn scrape(&mut self, client: &Client, ctx: &mut Context) -> Result<Html> {
        // TODO: use config
        let durations = delay::Fixed::from_millis(2000).take(3);
        let html = retry(durations, || self.retry_get(client, ctx))
            .map_err(|err| match err {
                retry::Error::Operation { error, .. } => error,
                retry::Error::Internal(msg) => Error::msg(msg),
            })
            .context("Could not get page from service")?;
        Ok(html)
    }

    fn retry_get(&self, client: &Client, ctx: &mut Context) -> OperationResult<Html, Error> {
        let url = self.url();
        write!(ctx.stderr, "{:6} {} ... ", "GET", url.as_str()).unwrap_or(());
        let req = client.get(url);
        let result = req.send();
        match &result {
            Ok(res) => writeln!(ctx.stderr, "{}", res.status()),
            Err(_) => writeln!(ctx.stderr, "failed"),
        }
        .unwrap_or(());
        let result = result
            .map_err(|err| err.into())
            .and_then(|res| {
                if self.is_acceptable(&res) {
                    res.text().map_err(|err| err.into())
                } else {
                    Err(Error::msg("Unacceptable response"))
                }
            })
            .and_then(|text| Ok(Html::parse_document(&text)));
        match result {
            Ok(html) => OperationResult::Ok(html),
            Err(err) => OperationResult::Retry(err),
        }
    }
}

pub trait Serve {
    fn login(&mut self, user: &str, pass: &str) -> Result<LoginOutcome>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginOutcome {
    service_id: ServiceKind,
    username: String,
}

impl fmt::Display for LoginOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Successfully logged in to {} as {}",
            Into::<&'static str>::into(&self.service_id),
            &self.username
        )
    }
}
