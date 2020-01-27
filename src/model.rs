use std::cmp::Ordering;
use std::convert::Infallible;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::redirect::Policy;
use reqwest::Url;
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
    #[serde(with = "string")]
    url: Url,
    samples: Vec<Sample>,
}

impl Problem {
    pub fn new(
        id: impl Into<ProblemId>,
        name: impl Into<String>,
        url: Url,
        samples: Vec<Sample>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url,
            samples,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct ProblemId(String);

impl ProblemId {
    pub fn normalize(&self) -> String {
        self.0.to_uppercase()
    }
}

impl PartialEq<ProblemId> for ProblemId {
    fn eq(&self, other: &ProblemId) -> bool {
        self.normalize() == other.normalize()
    }
}

impl PartialOrd for ProblemId {
    fn partial_cmp(&self, other: &ProblemId) -> Option<Ordering> {
        Some(self.normalize().cmp(&other.normalize()))
    }
}

impl Ord for ProblemId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.normalize().cmp(&other.normalize())
    }
}

impl Hash for ProblemId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().hash(state);
    }
}

impl<T: Into<String>> From<T> for ProblemId {
    fn from(id: T) -> Self {
        Self(id.into())
    }
}

impl FromStr for ProblemId {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl fmt::Display for ProblemId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

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

pub mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
