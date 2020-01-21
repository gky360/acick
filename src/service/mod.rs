use std::fmt;

use anyhow::Context as _;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::Url;
use retry::{delay, retry, OperationResult};
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::model::ServiceKind;
use crate::{Error, Result};

mod atcoder;

pub use atcoder::AtcoderService;

trait Accept<T> {
    fn is_acceptable(&self, data: &T) -> bool;
}

trait Scrape: Accept<Response> {
    const HOST: &'static str;
    const PATH: &'static str;

    fn url(&self) -> Url {
        Url::parse(Self::HOST).unwrap().join(Self::PATH).unwrap()
    }

    fn scrape(&mut self, client: &Client) -> Result<Html> {
        // TODO: use config
        let durations = delay::Fixed::from_millis(2000).take(3);
        let html = retry(durations, || {
            let url = self.url();
            let req = client.get(url);
            self.retry_get(req)
        })
        .map_err(|err| match err {
            retry::Error::Operation { error, .. } => error,
            retry::Error::Internal(msg) => Error::msg(msg),
        })
        .context("Could not get page from service")?;
        Ok(html)
    }

    fn retry_get(&self, req: RequestBuilder) -> OperationResult<Html, Error> {
        let result = req
            .send()
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
