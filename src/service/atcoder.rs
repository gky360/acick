use crate::service::{LoginOutcome, Serve};
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
