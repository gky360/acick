use anyhow::Context as _;
use reqwest::blocking::{Client, Request, RequestBuilder, Response};
use retry::{delay, retry, OperationResult};

use crate::{Context, Error, Result};

trait ExecSession {
    fn exec_session(&self, request: Request, ctx: &mut Context) -> Result<Response>;
}

impl ExecSession for Client {
    fn exec_session(&self, mut request: Request, ctx: &mut Context) -> Result<Response> {
        let mut storage = ctx
            .conf
            .open_cookie_storage()
            .context("Could not open cookie storage")?;
        storage
            .load_into(&mut request)
            .context("Could not load cookies into request")?;
        let response = self.execute(request)?;
        storage
            .store_from(&response)
            .context("Could not store cookies from response")?;
        Ok(response)
    }
}

pub struct RetryRequestBuilder<'a, 'b> {
    inner: RequestBuilder,
    client: &'a Client,
    ctx: &'a mut Context<'b>,
}

impl<'a, 'b> RetryRequestBuilder<'a, 'b> {
    pub fn send_pretty(&mut self) -> Result<Response> {
        let Self { client, ctx, .. } = self;
        let req = self
            .inner
            .try_clone()
            .ok_or_else(|| Error::msg("Could not build request"))?
            .build()?;
        write!(ctx.stderr, "{:7} {} ... ", req.method().as_str(), req.url()).unwrap_or(());
        let result = client
            .exec_session(req, ctx)
            .context("Could not send request");
        match &result {
            Ok(res) => writeln!(ctx.stderr, "{}", res.status()),
            Err(_) => writeln!(ctx.stderr, "failed"),
        }
        .unwrap_or(());
        result
    }

    pub fn retry_send(&mut self) -> Result<Response> {
        let session = self.ctx.conf.body().session();
        let retry_interval = session.retry_interval().as_millis() as u64;
        let retry_limit = session.retry_limit();
        let durations = delay::Fixed::from_millis(retry_interval).take(retry_limit);
        retry(durations, || match self.send_pretty() {
            Ok(res) => {
                if res.status().is_server_error() {
                    OperationResult::Retry(Error::msg("Received server error"))
                } else {
                    OperationResult::Ok(res)
                }
            }
            Err(err) => OperationResult::Retry(err),
        })
        .map_err(|err| match err {
            retry::Error::Operation { error, .. } => error,
            retry::Error::Internal(msg) => Error::msg(msg),
        })
    }
}

pub trait WithRetry {
    fn with_retry<'a, 'b>(
        self,
        client: &'a Client,
        ctx: &'a mut Context<'b>,
    ) -> RetryRequestBuilder<'a, 'b>;
}

impl WithRetry for RequestBuilder {
    fn with_retry<'a, 'b>(
        self,
        client: &'a Client,
        ctx: &'a mut Context<'b>,
    ) -> RetryRequestBuilder<'a, 'b> {
        RetryRequestBuilder {
            inner: self,
            client,
            ctx,
        }
    }
}
