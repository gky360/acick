use std::cmp::max;
use std::fmt;
use std::io::Write as _;
use std::time::Duration;

use console::StyledObject;
use getset::CopyGetters;
use serde::{Deserialize, Serialize};

use crate::console::{
    sty_dim, sty_g, sty_g_rev, sty_g_under, sty_none, sty_r, sty_r_rev, sty_r_under, sty_y,
    sty_y_rev, sty_y_under,
};
use crate::judge::diff::TextDiff;
use crate::{Console, Error, Result};

#[derive(
    Serialize, Deserialize, AsRefStr, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum StatusKind {
    #[strum(serialize = " A C ")]
    Ac,
    #[strum(serialize = " W A ")]
    Wa,
    #[strum(serialize = " TLE ")]
    Tle,
    #[strum(serialize = " R E ")]
    Re,
}

impl StatusKind {
    fn sty<D>(self, val: D) -> StyledObject<D> {
        match self {
            Self::Ac => sty_g(val),
            Self::Wa => sty_r(val),
            Self::Tle => sty_y(val),
            Self::Re => sty_y(val),
        }
    }

    fn sty_under<D>(self, val: D) -> StyledObject<D> {
        match self {
            Self::Ac => sty_g_under(val),
            Self::Wa => sty_r_under(val),
            Self::Tle => sty_y_under(val),
            Self::Re => sty_y_under(val),
        }
    }

    fn sty_under_if<D>(self, val: D, condition: bool) -> StyledObject<D> {
        if condition {
            self.sty_under(val)
        } else {
            sty_none(val)
        }
    }

    fn sty_rev<D>(self, val: D) -> StyledObject<D> {
        match self {
            Self::Ac => sty_g_rev(val),
            Self::Wa => sty_r_rev(val),
            Self::Tle => sty_y_rev(val),
            Self::Re => sty_y_rev(val),
        }
    }
}

impl fmt::Display for StatusKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.sty_rev(self.as_ref()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE", tag = "kind")]
enum StatusInner {
    Ac,
    Wa { diff: TextDiff },
    Tle,
    Re { reason: String },
}

impl StatusInner {
    fn describe(&self, cnsl: &mut Console) -> Result<()> {
        match self {
            Self::Ac => {}
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
    pub fn ac(sample_name: String, elapsed: Duration) -> Self {
        Self {
            sample_name,
            elapsed,
            inner: StatusInner::Ac,
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
        let elapsed = format!("({:>4}ms)", self.elapsed.as_millis());
        let elapsed = if self.kind() == StatusKind::Tle {
            StatusKind::Tle.sty(elapsed)
        } else {
            sty_dim(elapsed)
        };
        write!(f, "{} {}", self.kind(), elapsed)
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

#[derive(Serialize, Deserialize, CopyGetters, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TotalStatus {
    #[get_copy = "pub"]
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

    pub fn count(&self) -> usize {
        self.count.total()
    }
}

impl fmt::Display for TotalStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let StatusCount { ac, wa, tle, re } = self.count;
        write!(
            f,
            "{} (AC: {:>2}/{t:>2}, WA: {:>2}/{t:>2}, TLE: {:>2}/{t:>2}, RE: {:>2}/{t:>2})",
            self.kind,
            ac,
            StatusKind::Wa.sty_under_if(wa, wa > 0),
            StatusKind::Tle.sty_under_if(tle, tle > 0),
            StatusKind::Re.sty_under_if(re, re > 0),
            t = self.count.total()
        )
    }
}
