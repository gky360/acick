use std::time::Duration;

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

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

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            retry_limit: 4,
            retry_interval: Duration::from_secs(2),
        }
    }
}
