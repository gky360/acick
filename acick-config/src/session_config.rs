use std::time::Duration;

use getset::{CopyGetters, Getters};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::abs_path::AbsPathBuf;
use crate::DATA_LOCAL_DIR;

static COOKIES_FILE_NAME: &str = "cookies.json";

lazy_static! {
    static ref COOKIES_PATH: AbsPathBuf = DATA_LOCAL_DIR.join(COOKIES_FILE_NAME);
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_RETRY_LIMIT: usize = 4;
const DEFAULT_RETRY_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Serialize, Deserialize, Getters, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(default)]
pub struct SessionConfig {
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    timeout: Duration,
    #[serde(skip, default = "SessionConfig::default_cookies_path")]
    #[get = "pub"]
    cookies_path: AbsPathBuf,
    #[get_copy = "pub"]
    retry_limit: usize,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    retry_interval: Duration,
}

impl SessionConfig {
    pub fn default_in_dir(base_dir: &AbsPathBuf) -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            cookies_path: base_dir.join(COOKIES_FILE_NAME),
            retry_limit: DEFAULT_RETRY_LIMIT,
            retry_interval: DEFAULT_RETRY_INTERVAL,
        }
    }

    fn default_cookies_path() -> AbsPathBuf {
        COOKIES_PATH.clone()
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            cookies_path: Self::default_cookies_path(),
            retry_limit: DEFAULT_RETRY_LIMIT,
            retry_interval: DEFAULT_RETRY_INTERVAL,
        }
    }
}
