use anyhow::Context as _;
use reqwest::blocking::Response;
use reqwest::header::LOCATION;
use reqwest::Url;

use crate::Result;

mod act;
mod cookie;
mod scrape;
mod session;

pub use self::cookie::CookieStorage;
pub use act::Act;
// pub use atcoder::AtcoderActor;

// use crate::model::ServiceKind;

// pub fn with_actor<F, R>(service_id: ServiceKind, session: &SessionConfig, f: F) -> R
// where
//     F: FnOnce(&dyn Act) -> R,
// {
//     match service_id {
//         ServiceKind::Atcoder => f(&AtcoderActor::new(session)),
//     }
// }

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
