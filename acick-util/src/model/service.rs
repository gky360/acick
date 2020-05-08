use std::fmt;
use std::hash::Hash;

use getset::CopyGetters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Service {
    #[get_copy = "pub"]
    id: ServiceKind,
}

impl Service {
    pub fn new(id: ServiceKind) -> Self {
        Self { id }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new(ServiceKind::default())
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
    PartialOrd,
    Ord,
    Hash,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ServiceKind {
    Atcoder,
}

impl ServiceKind {
    pub fn to_user_pass_env_names(self) -> (&'static str, &'static str) {
        match self {
            Self::Atcoder => ("ACICK_ATCODER_USERNAME", "ACICK_ATCODER_PASSWORD"),
        }
    }
}

impl Default for ServiceKind {
    fn default() -> Self {
        Self::Atcoder
    }
}

impl fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_kind_default_display() {
        assert_eq!(ServiceKind::default().to_string(), "atcoder");
    }
}
