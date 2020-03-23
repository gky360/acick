use anyhow::Context as _;
use reqwest::blocking::Response;
use reqwest::header::LOCATION;
use reqwest::Url;

use crate::Result;

pub mod act;
mod cookie;
pub mod scrape;
pub mod session;

pub use self::cookie::CookieStorage;
pub use act::Act;

pub trait ResponseExt {
    fn location_url(&self, base: &Url) -> Result<Url>;
}

impl ResponseExt for Response {
    fn location_url(&self, base: &Url) -> Result<Url> {
        let loc_str = self
            .headers()
            .get(LOCATION)
            .context("Could not find location header in response")?
            .to_str()?;
        base.join(loc_str)
            .context("Could not parse redirection url")
    }
}
