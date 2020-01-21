use reqwest::blocking::Response;

use crate::service::{Accept, LoginOutcome, Scrape, Serve};
use crate::{Config, Context, GlobalOpt, Input, Output, Result};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AtcoderService<'a, I: Input, O: Output, E: Output> {
    global_opt: &'a GlobalOpt,
    conf: &'a Config,
    ctx: &'a mut Context<I, O, E>,
}

impl<'a, I: Input, O: Output, E: Output> AtcoderService<'a, I, O, E> {
    pub fn new(global_opt: &'a GlobalOpt, conf: &'a Config, ctx: &'a mut Context<I, O, E>) -> Self {
        Self {
            global_opt,
            conf,
            ctx,
        }
    }
}

impl<'a, I: Input, O: Output, E: Output> Serve for AtcoderService<'a, I, O, E> {
    fn login(&mut self, user: &str, _pass: &str) -> Result<LoginOutcome> {
        // TODO: login

        let outcome = LoginOutcome {
            service_id: self.global_opt.service_id.clone(),
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
