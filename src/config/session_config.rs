use std::time::Duration;

use dirs::{data_local_dir, home_dir};
use getset::{CopyGetters, Getters};
use lazy_static::lazy_static;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};

use crate::abs_path::AbsPathBuf;
use crate::service::CookieStorage;
use crate::Result;

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "-",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    ")"
);

static COOKIES_FILE_NAME: &str = "cookies.json";

lazy_static! {
    static ref COOKIES_PATH: AbsPathBuf = {
        let path = data_local_dir()
            .unwrap_or_else(|| {
                home_dir()
                    .expect("Could not get home dir")
                    .join(".local")
                    .join("share")
            })
            .join(env!("CARGO_PKG_NAME"))
            .join(COOKIES_FILE_NAME);
        AbsPathBuf::try_new(path).unwrap()
    };
}

#[derive(Serialize, Deserialize, Getters, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct SessionConfig {
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    timeout: Duration,
    #[get_copy = "pub"]
    retry_limit: usize,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    retry_interval: Duration,
}

impl SessionConfig {
    pub fn open_cookie_storage(&self) -> Result<CookieStorage> {
        CookieStorage::open(&COOKIES_PATH)
    }

    pub fn get_client_builder(&self) -> ClientBuilder {
        // TODO : switch client by service
        Client::builder()
            .referer(false)
            .redirect(Policy::none()) // redirects manually
            .user_agent(USER_AGENT)
            .timeout(Some(self.timeout))
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_limit: 4,
            retry_interval: Duration::from_secs(2),
        }
    }
}
