use std::fmt;

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
    #[allow(dead_code)]
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

impl fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contest {
    id: ContestId,
    name: String,
    problems: Vec<Problem>,
}

impl Contest {
    pub fn new(id: impl Into<ContestId>, name: impl Into<String>, problems: Vec<Problem>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            problems,
        }
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
    pub fn new(id: impl Into<ProblemId>, name: impl Into<String>, samples: Vec<Sample>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            samples,
        }
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
    pub fn new(
        name: impl Into<String>,
        input: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input: input.into(),
            output: output.into(),
        }
    }
}
