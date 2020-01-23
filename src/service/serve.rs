use std::fmt;

use serde::{Deserialize, Serialize};

use crate::model::ServiceKind;
use crate::Result;

pub trait Serve {
    fn login(&mut self, user: String, pass: String) -> Result<LoginOutcome>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginOutcome {
    pub service_id: ServiceKind,
    pub username: String,
}

impl fmt::Display for LoginOutcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Successfully logged in to {} as {}",
            Into::<&'static str>::into(&self.service_id),
            &self.username
        )
    }
}
