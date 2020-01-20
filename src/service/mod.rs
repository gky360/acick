use std::fmt;

use reqwest::blocking::Client;
use reqwest::Url;
use scraper::Html;
use serde::{Deserialize, Serialize};

use crate::model::ServiceKind;
use crate::Result;

mod atcoder;

pub use atcoder::AtcoderService;

pub trait Scrape {
    fn url() -> Url;

    fn fetch(client: &Client) -> Result<Html>;
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
