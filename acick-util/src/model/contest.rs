use std::cmp::Ordering;
use std::convert::Infallible;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::regex;

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

impl Default for Contest {
    fn default() -> Self {
        Self::new(DEFAULT_CONTEST_ID_STR, "AtCoder Regular Contest 100")
    }
}

pub static DEFAULT_CONTEST_ID_STR: &str = "arc100";

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct ContestId(String);

impl ContestId {
    pub fn normalize(&self) -> String {
        regex!(r"[-_]").replace_all(&self.0, "").to_lowercase()
    }
}

impl Default for ContestId {
    fn default() -> Self {
        Self::from(DEFAULT_CONTEST_ID_STR)
    }
}

impl PartialEq<ContestId> for ContestId {
    fn eq(&self, other: &ContestId) -> bool {
        self.normalize() == other.normalize()
    }
}

impl PartialOrd for ContestId {
    fn partial_cmp(&self, other: &ContestId) -> Option<Ordering> {
        Some(self.normalize().cmp(&other.normalize()))
    }
}

impl Ord for ContestId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.normalize().cmp(&other.normalize())
    }
}

impl Hash for ContestId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().hash(state);
    }
}

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

impl AsRef<str> for ContestId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContestId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contest_id_eq() {
        assert_eq!(ContestId::from("arc100"), ContestId::from("arc100"));
        assert_eq!(ContestId::from("ARC100"), ContestId::from("arc100"));
        assert_eq!(
            ContestId::from("CodeFestival2017QualA"),
            ContestId::from("code-festival-2017-quala")
        );
    }
}
