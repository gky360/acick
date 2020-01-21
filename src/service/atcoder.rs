use reqwest::blocking::Response;

use crate::service::{Accept, LoginOutcome, Scrape, Serve};
use crate::{Context, Result};

#[derive(Debug)]
pub struct AtcoderService<'a, 'b> {
    ctx: &'a mut Context<'b>,
}

impl<'a, 'b> AtcoderService<'a, 'b> {
    pub fn new(ctx: &'a mut Context<'b>) -> Self {
        Self { ctx }
    }
}

impl Serve for AtcoderService<'_, '_> {
    fn login(&mut self, user: &str, _pass: &str) -> Result<LoginOutcome> {
        // TODO: login

        let outcome = LoginOutcome {
            service_id: self.ctx.global_opt.service_id.clone(),
            username: user.to_string(),
        };
        Ok(outcome)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoginPage {}

impl Accept<Response> for LoginPage {
    fn is_acceptable(&self, res: &Response) -> bool {
        res.status().is_success()
    }
}

impl Scrape for LoginPage {
    const HOST: &'static str = "https://atcoder.jp";
    const PATH: &'static str = "/login";
}
