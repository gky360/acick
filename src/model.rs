use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};

use crate::service::{AtcoderService, Serve};
use crate::Context;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Service {
    id: ServiceKind,
}

impl Service {
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

    fn get_client_builder(self, ctx: &mut Context) -> ClientBuilder {
        let session = ctx.conf.data().session();
        let user_agent = session.user_agent();
        let timeout = session.timeout();
        Client::builder()
            .referer(false)
            .redirect(Policy::none()) // redirects manually
            .user_agent(user_agent)
            .timeout(Some(timeout))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contest {
    id: ContestId,
}

impl Contest {
    pub fn new(id: ContestId) -> Self {
        Self { id }
    }
}

pub type ContestId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Problem {
    id: ProblemId,
    name: String,
    samples: Vec<Sample>,
}

impl Problem {
    pub fn new(id: ProblemId, name: String, samples: Vec<Sample>) -> Self {
        Self { id, name, samples }
    }
}

pub type ProblemId = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sample {
    name: String,
    input: String,
    output: String,
}

impl Sample {
    pub fn new(name: String, input: String, output: String) -> Self {
        Self {
            name,
            input,
            output,
        }
    }
}
