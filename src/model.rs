use serde::{Deserialize, Serialize};

use crate::service::{AtcoderService, Serve};
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
    pub fn serve<'a>(&self, ctx: &'a mut Context<'_>) -> Box<dyn Serve + 'a> {
        match self {
            Self::Atcoder => Box::new(AtcoderService::new(ctx)),
        }
    }

    pub fn to_user_pass_env_names(&self) -> (&'static str, &'static str) {
        match self {
            Self::Atcoder => ("ACICK_ATCODER_USERNAME", "ACICK_ATCODER_PASSWORD"),
        }
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
