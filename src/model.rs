use std::cmp::Ordering;
use std::convert::{Infallible, TryFrom};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::Duration;

use getset::{CopyGetters, Getters, Setters};
use reqwest::Url;
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

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
pub struct Contest {
    id: ContestId,
    name: String,
}

impl Contest {
    pub fn new(id: impl Into<ContestId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContestId(String);

impl<T: Into<String>> From<T> for ContestId {
    fn from(id: T) -> Self {
        Self(id.into())
    }
}

impl FromStr for ContestId {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl fmt::Display for ContestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(
    Serialize, Deserialize, Getters, CopyGetters, Setters, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct Problem {
    #[get = "pub"]
    id: ProblemId,
    #[get = "pub"]
    name: String,
    #[serde(with = "string")]
    #[get = "pub"]
    url: Url,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    time_limit: Duration,
    #[get_copy = "pub"]
    memory_limit: Byte,
    #[get_copy = "pub"]
    compare: Compare,
    #[set = "pub"]
    #[get = "pub"]
    samples: Vec<Sample>,
}

impl Problem {
    pub fn new(
        id: impl Into<ProblemId>,
        name: impl Into<String>,
        url: Url,
        time_limit: Duration,
        memory_limit: Byte,
        compare: Compare,
        samples: Vec<Sample>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url,
            time_limit,
            memory_limit,
            compare,
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
pub enum Compare {
    Exact,
    // TODO: support float
    // Float {
    //     relative_error: Option<f64>,
    //     absolute_error: Option<f64>,
    // },
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct Byte(u64);

impl FromStr for Byte {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(bytefmt::parse(s)?))
    }
}

impl TryFrom<String> for Byte {
    type Error = &'static str;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl From<Byte> for String {
    fn from(byte: Byte) -> Self {
        bytefmt::format_to(byte.0, bytefmt::Unit::MB)
    }
}

impl fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&String::from(*self))
    }
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, Eq, Hash)]
#[get = "pub"]
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
