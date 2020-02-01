use std::fmt;
use std::io::Write as _;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::judge::diff::TextDiff;
use crate::{Console, Error, Result};

#[derive(
    Serialize,
    Deserialize,
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
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum StatusKind {
    Ac,
    Wa,
    Tle,
    Re,
}

impl fmt::Display for StatusKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE", tag = "kind")]
enum StatusInner {
    Ac { diff: TextDiff },
    Wa { diff: TextDiff },
    Tle,
    Re { reason: String },
}

impl StatusInner {
    fn describe(&self, cnsl: &mut Console) -> Result<()> {
        match self {
            Self::Ac { .. } => {}
            Self::Wa { diff } => writeln!(cnsl, "{}", diff)?,
            Self::Tle => {}
            Self::Re { reason } => writeln!(cnsl, "{}", reason)?,
        }
        Ok(())
    }

    fn to_kind(&self) -> StatusKind {
        match self {
            Self::Ac { .. } => StatusKind::Ac,
            Self::Wa { .. } => StatusKind::Wa,
            Self::Tle => StatusKind::Tle,
            Self::Re { .. } => StatusKind::Re,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Status {
    sample_name: String,
    #[serde(with = "humantime_serde")]
    elapsed: Duration,
    #[serde(flatten)]
    inner: StatusInner,
}

impl Status {
    pub fn ac(sample_name: String, elapsed: Duration, diff: TextDiff) -> Self {
        Self {
            sample_name,
            elapsed,
            inner: StatusInner::Ac { diff },
        }
    }

    pub fn wa(sample_name: String, elapsed: Duration, diff: TextDiff) -> Self {
        Self {
            sample_name,
            elapsed,
            inner: StatusInner::Wa { diff },
        }
    }

    pub fn tle(sample_name: String, elapsed: Duration) -> Self {
        Self {
            sample_name,
            elapsed,
            inner: StatusInner::Tle,
        }
    }

    pub fn re(sample_name: String, elapsed: Duration, err: Error) -> Self {
        Self {
            sample_name,
            elapsed,
            inner: StatusInner::Re {
                reason: format!("{:?}\n", err),
            },
        }
    }

    pub fn kind(&self) -> StatusKind {
        self.inner.to_kind()
    }

    pub fn describe(&self, cnsl: &mut Console) -> Result<()> {
        self.inner.describe(cnsl)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}ms)", self.kind(), self.elapsed.as_millis())
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
// pub struct TotalStatus {
//     kind: TotalStatusKind,
//     statuses: Vec<Status>,
// }

// impl TotalStatus {
//     pub fn new(statuses: Vec<Status>) -> Self {

//     }
// }

// #[derive(
//     Serialize, Deserialize, EnumVariantNames, IntoStaticStr, Debug, Copy, Clone, PartialEq, Eq, Hash,
// )]
// #[serde(rename_all = "UPPERCASE")]
// #[strum(serialize_all = "UPPERCASE")]
// pub enum TotalStatusKind {
//     Ac,
//     Wa,
//     Tle,
//     Re,
// }
