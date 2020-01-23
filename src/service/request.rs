use anyhow::Context as _;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::StatusCode;
use retry::{delay, retry, OperationResult};

use crate::{Context, Error, Result};

pub struct RetryRequestBuilder<'a, 'b> {
    inner: RequestBuilder,
    is_accept: Box<dyn Fn(StatusCode) -> bool + 'a>,
    is_reject: Box<dyn Fn(StatusCode) -> bool + 'a>,
    client: &'a Client,
    ctx: &'a mut Context<'b>,
}

impl<'a, 'b> RetryRequestBuilder<'a, 'b> {
    pub fn accept(mut self, accept: impl Fn(StatusCode) -> bool + 'a) -> Self {
        self.is_accept = Box::new(accept);
        self
    }

    pub fn reject(mut self, reject: impl Fn(StatusCode) -> bool + 'a) -> Self {
        self.is_reject = Box::new(reject);
        self
    }

    pub fn send(&mut self) -> Result<Response> {
        let Self { client, ctx, .. } = self;
        let req = self
            .inner
            .try_clone()
            .ok_or_else(|| Error::msg("Could not build request"))?
            .build()?;
        write!(ctx.stderr, "{:7} {} ... ", req.method().as_str(), req.url()).unwrap_or(());
        let result = client.execute(req).context("Could not send request");
        match &result {
            Ok(res) => writeln!(ctx.stderr, "{}", res.status()),
            Err(_) => writeln!(ctx.stderr, "failed"),
        }
        .unwrap_or(());
        result
    }

    pub fn retry_send(&mut self) -> Result<Option<Response>> {
        // TODO: use config
        let durations = delay::Fixed::from_millis(1000).take(4);
        retry(durations, || match self.send() {
            Ok(res) => {
                if self.is_accept.as_ref()(res.status()) {
                    OperationResult::Ok(Some(res))
                } else if self.is_reject.as_ref()(res.status()) {
                    OperationResult::Ok(None)
                } else {
                    OperationResult::Retry(Error::msg("Received request needs retry"))
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
            is_accept: Box::new(|status: StatusCode| status.is_success()),
            is_reject: Box::new(|status: StatusCode| {
                status.is_redirection() || status.is_client_error()
            }),
            client,
            ctx,
        }
    }
}
