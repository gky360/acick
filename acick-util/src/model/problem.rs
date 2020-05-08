use std::cmp::Ordering;
use std::convert::{Infallible, TryFrom};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::time::Duration;

use getset::{CopyGetters, Getters, Setters};
use serde::{Deserialize, Serialize};

use crate::model::sample::{Sample, SampleIter};

#[derive(
    Serialize, Deserialize, Getters, CopyGetters, Setters, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct Problem {
    #[get = "pub"]
    id: ProblemId,
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    url_name: String,
    #[serde(with = "humantime_serde")]
    #[get_copy = "pub"]
    time_limit: Duration,
    #[get_copy = "pub"]
    memory_limit: Byte,
    #[get_copy = "pub"]
    compare: Compare,
    #[set = "pub"]
    samples: Vec<Sample>,
}

impl Problem {
    pub fn new(
        id: impl Into<ProblemId>,
        name: impl Into<String>,
        url_name: impl Into<String>,
        time_limit: Duration,
        memory_limit: Byte,
        compare: Compare,
        samples: Vec<Sample>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            url_name: url_name.into(),
            time_limit,
            memory_limit,
            compare,
            samples,
        }
    }

    pub fn take_samples(self, sample_name: &Option<String>) -> SampleIter {
        if let Some(sample_name) = sample_name {
            self.samples
                .into_iter()
                .filter(|sample| sample.name() == sample_name)
                .collect::<Vec<_>>()
                .into()
        } else {
            self.samples.into()
        }
    }
}

impl Default for Problem {
    fn default() -> Self {
        Self::new(
            "C",
            "Linear Approximation",
            "arc100_a",
            Duration::from_secs(2),
            "1024 MB".parse().unwrap(),
            Compare::Default,
            vec![],
        )
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

impl AsRef<str> for ProblemId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProblemId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.normalize())
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
    Default,
    // TODO: support float
    // Float {
    //     relative_error: Option<f64>,
    //     absolute_error: Option<f64>,
    // },
}

impl Compare {
    pub fn compare(self, a: &str, b: &str) -> bool {
        match self {
            Self::Default => Self::compare_default(a, b),
        }
    }

    fn compare_default(a: &str, b: &str) -> bool {
        a.trim_end() == b.trim_end() // ignore spaces at the end of lines
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn problem_id_eq() {
        assert_eq!(ProblemId::from("A"), ProblemId::from("A"));
        assert_eq!(ProblemId::from("a"), ProblemId::from("A"));
    }
}
