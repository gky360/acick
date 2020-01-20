use crate::service::Serve;
use crate::{Context, Input, Output, Result};

pub struct AtcoderService<'a, I: Input, O: Output, E: Output> {
    ctx: &'a mut Context<I, O, E>,
}

impl<'a, I: Input, O: Output, E: Output> AtcoderService<'a, I, O, E> {
    pub fn new(ctx: &'a mut Context<I, O, E>) -> Self {
        Self { ctx }
    }
}

impl<'a, I: Input, O: Output, E: Output> Serve for AtcoderService<'a, I, O, E> {
    fn login(&mut self, user: &str, pass: &str) -> Result<()> {
        writeln!(self.ctx.stderr, "{:?}", (user, pass))?;
        Ok(())
    }
}
