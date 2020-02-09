use std::io::Write as _;

use anyhow::Context as _;
use reqwest::blocking::Response;
use reqwest::header::LOCATION;
use reqwest::Url;

use crate::{Console, Error, Result};

mod act;
mod atcoder;
mod atcoder_page;
mod cookie;
mod dropbox;
mod scrape;
mod session;

pub use self::cookie::CookieStorage;
pub use act::Act;
pub use atcoder::AtcoderActor;

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

fn open_in_browser(url: &str, cnsl: &mut Console) -> Result<()> {
    if cfg!(test) {
        unreachable!("Cannot open url in browser during test");
    }
    match webbrowser::open(url) {
        Err(err) => Err(err.into()),
        Ok(output) if !output.status.success() => {
            Err(Error::msg("Process returned non-zero exit code"))
        }
        _ => Ok(writeln!(cnsl, "Opened in browser : {}", url)?),
    }
    .with_context(|| format!("Could not open url in browser : {}", url))
}
