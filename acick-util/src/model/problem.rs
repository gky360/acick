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
    time_limit: Option<Duration>,
    #[get_copy = "pub"]
    memory_limit: Option<Byte>,
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
        time_limit: Option<Duration>,
        memory_limit: Option<Byte>,
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
            Some(Duration::from_secs(2)),
            Some("1024 MB".parse().unwrap()),
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
    fn test_problem_take_sapmles() {
        let samples = vec![
            Sample::new("name 1", "5", "0"),
            Sample::new("name 2", "5", "0"),
        ];
        let problem = Problem {
            id: "A".into(),
            name: "Problem A".into(),
            url_name: "test_contest_a".into(),
            time_limit: Some(Duration::from_secs(2)),
            memory_limit: Some("1024 KB".parse().unwrap()),
            compare: Compare::Default,
            samples: samples.clone(),
        };
        let tests = &[
            (Some(String::from("name 2")), vec![&samples[1]]),
            (None, vec![&samples[0], &samples[1]]),
        ];

        for (sample_name, expected) in tests {
            let actual = &problem
                .clone()
                .take_samples(&sample_name)
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), expected.len());
            let is_all_equal = actual
                .iter()
                .zip(expected)
                .all(|(a, b)| a.as_ref().unwrap() == *b);
            assert!(is_all_equal);
        }
    }

    #[test]
    fn problem_id_eq() {
        assert_eq!(ProblemId::from("A"), ProblemId::from("A"));
        assert_eq!(ProblemId::from("a"), ProblemId::from("A"));
    }

    #[test]
    fn test_problem_id_display() {
        assert_eq!(&ProblemId::from("A").to_string(), "A");
        assert_eq!(&ProblemId::from("a").to_string(), "A");
    }

    #[test]
    fn test_compare() {
        let tests = &[
            (Compare::Default, "hoge", "hoge", true),
            (Compare::Default, "hoge", "hoge  ", true),
            (Compare::Default, "hoge", "hoge\n", true),
            (Compare::Default, "hoge", "  hoge", false),
            (Compare::Default, "hoge", "\nhoge", false),
        ];

        for (compare, a, b, expected) in tests {
            let actual = compare.compare(a, b);
            assert_eq!(actual, *expected);
        }
    }

    #[test]
    fn test_byte_try_from() -> anyhow::Result<()> {
        assert_eq!(
            Byte::try_from(String::from("1024KB")).unwrap(),
            Byte(1024 * 1000)
        );
        assert_eq!(
            Byte::try_from(String::from("1.2MB")).unwrap(),
            Byte(1200 * 1000)
        );
        Ok(())
    }

    #[test]
    fn test_byte_display() {
        assert_eq!(&Byte(1024 * 1000).to_string(), "1.02 MB");
        assert_eq!(&Byte(2000 * 1000).to_string(), "2 MB");
        assert_eq!(&Byte(10 * 1000 * 1000).to_string(), "10 MB");
    }
}
