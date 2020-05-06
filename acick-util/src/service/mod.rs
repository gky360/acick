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

#[cfg(test)]
mod tests {
    use reqwest::blocking::Client;
    use reqwest::redirect::Policy;

    use super::*;

    #[test]
    fn test_location_url() -> anyhow::Result<()> {
        let client = Client::builder()
            .redirect(Policy::none()) // redirects manually
            .build()
            .unwrap();
        let res = client.get("https://mail.google.com").send()?;
        let actual = res.location_url(&Url::parse("https://mail.google.com").unwrap())?;
        let expected = Url::parse("https://mail.google.com/mail/").unwrap();
        assert_eq!(actual, expected);
        Ok(())
    }
}
