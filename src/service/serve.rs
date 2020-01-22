use std::fmt;

use anyhow::Context as _;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};

use crate::model::ServiceKind;
use crate::{Context, Result};

pub trait SendPretty {
    fn send_pretty(self, client: &Client, ctx: &mut Context) -> Result<Response>;
}

impl SendPretty for RequestBuilder {
    fn send_pretty(self, client: &Client, ctx: &mut Context) -> Result<Response> {
        let req = self.build()?;
        write!(ctx.stderr, "{:7} {} ... ", req.method().as_str(), req.url()).unwrap_or(());
        let result = client.execute(req).context("Could not send request");
        match &result {
            Ok(res) => writeln!(ctx.stderr, "{}", res.status()),
            Err(_) => writeln!(ctx.stderr, "failed"),
        }
        .unwrap_or(());
        result
    }
}

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
