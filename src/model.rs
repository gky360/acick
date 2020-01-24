use std::time::Duration;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};

use crate::service::{AtcoderService, Serve, USER_AGENT};
use crate::Context;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Service {
    id: ServiceKind,
}

impl Service {
    #[cfg(test)] // TODO: not only test
    pub fn new(id: ServiceKind) -> Self {
        Self { id }
    }
}

#[derive(
    Serialize,
    Deserialize,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ServiceKind {
    Atcoder,
}

impl ServiceKind {
    pub fn serve<'a>(self, ctx: &'a mut Context<'_>) -> Box<dyn Serve + 'a> {
        let client = self
            .get_client_builder(ctx)
            .build()
            .expect("Could not setup client. \
                TLS backend cannot be initialized, or the resolver cannot load the system configuration.");
        match self {
            Self::Atcoder => Box::new(AtcoderService::new(client, ctx)),
        }
    }

    pub fn to_user_pass_env_names(self) -> (&'static str, &'static str) {
        match self {
            Self::Atcoder => ("ACICK_ATCODER_USERNAME", "ACICK_ATCODER_PASSWORD"),
        }
    }

    fn get_client_builder(self, _ctx: &mut Context) -> ClientBuilder {
        Client::builder()
            .referer(false)
            .redirect(Policy::none()) // redirects manually
            .cookie_store(true) // TODO: use own cookie store
            .user_agent(USER_AGENT) // TODO: use config
            .timeout(Some(Duration::from_secs(30))) // TODO: use config
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contest {
    id: String,
}

impl Contest {
    #[cfg(test)] // TODO: not only test
    pub fn new(id: impl ToString) -> Self {
        Self { id: id.to_string() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Problem {
    id: String,
}

impl Problem {
    #[cfg(test)] // TODO: not only test
    pub fn new(id: impl ToString) -> Self {
        Self { id: id.to_string() }
    }
}
