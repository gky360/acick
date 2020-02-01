use std::fmt;
use std::io::Write as _;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::judge::diff::TextDiff;
use crate::{Console, Error, Result};

#[derive(
    Serialize, Deserialize, EnumVariantNames, IntoStaticStr, Debug, Clone, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum StatusKind {
    Ac { diff: TextDiff },
    Wa { diff: TextDiff },
    Tle,
    Re { reason: String },
}

impl StatusKind {
    pub fn ac(diff: TextDiff) -> Self {
        Self::Ac { diff }
    }

    pub fn wa(diff: TextDiff) -> Self {
        Self::Wa { diff }
    }

    pub fn re(err: Error) -> Self {
        Self::Re {
            reason: format!("{:?}\n", err),
        }
    }

    pub fn describe(&self, cnsl: &mut Console) -> Result<()> {
        match self {
            Self::Ac { .. } => {}
            Self::Wa { diff } => writeln!(cnsl, "{}", diff)?,
            Self::Tle => {}
            Self::Re { reason } => writeln!(cnsl, "{}", reason)?,
        }
        Ok(())
    }
}

impl fmt::Display for StatusKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Status {
    pub kind: StatusKind,
    pub sample_name: String,
    pub elapsed: Duration,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}ms)", self.kind, self.elapsed.as_millis())
    }
}
