use std::cmp::max;
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
struct StatusCount {
    ac: usize,
    wa: usize,
    tle: usize,
    re: usize,
}

impl StatusCount {
    fn new() -> Self {
        Self {
            ac: 0,
            wa: 0,
            tle: 0,
            re: 0,
        }
    }

    fn add(&mut self, kind: StatusKind) -> &mut Self {
        match kind {
            StatusKind::Ac => self.ac += 1,
            StatusKind::Wa => self.wa += 1,
            StatusKind::Tle => self.tle += 1,
            StatusKind::Re => self.re += 1,
        }
        self
    }

    fn total(&self) -> usize {
        self.ac + self.wa + self.tle + self.re
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TotalStatus {
    kind: StatusKind,
    count: StatusCount,
    statuses: Vec<Status>,
}

impl TotalStatus {
    pub fn new(statuses: Vec<Status>) -> Self {
        let (kind, count) = statuses.iter().fold(
            (StatusKind::Ac, StatusCount::new()),
            |(kind, mut count), status| {
                count.add(status.kind());
                (max(kind, status.kind()), count)
            },
        );

        Self {
            kind,
            count,
            statuses,
        }
    }
}

impl fmt::Display for TotalStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:3} (AC: {:>2}/{t:>2}, WA: {:>2}/{t:>2}, TLE: {:>2}/{t:>2}, RE: {:>2}/{t:>2})",
            Into::<&'static str>::into(self.kind),
            self.count.ac,
            self.count.wa,
            self.count.tle,
            self.count.re,
            t = self.count.total()
        )
    }
}
