use std::io::Write as _;

use anyhow::Context as _;
use reqwest::blocking::{Client, Request, RequestBuilder, Response};
use retry::{delay, retry, OperationResult};

use crate::config::{SessionConfig, COOKIES_PATH};
use crate::service::CookieStorage;
use crate::{Console, Error, Result};

trait ExecSession {
    fn exec_session(&self, request: Request) -> Result<Response>;
}

impl ExecSession for Client {
    fn exec_session(&self, mut request: Request) -> Result<Response> {
        let mut storage =
            CookieStorage::open(&COOKIES_PATH).context("Could not open cookie storage")?;
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

pub struct RetryRequestBuilder<'a> {
    inner: RequestBuilder,
    client: &'a Client,
    session: &'a SessionConfig,
    cnsl: &'a mut Console,
}

impl<'a> RetryRequestBuilder<'a> {
    pub fn send_pretty(&mut self) -> Result<Response> {
        let Self { client, cnsl, .. } = self;
        let req = self
            .inner
            .try_clone()
            .ok_or_else(|| Error::msg("Could not build request"))?
            .build()?;
        write!(cnsl, "{:7} {} ... ", req.method().as_str(), req.url()).unwrap_or(());
        let result = client.exec_session(req).context("Could not send request");
        match &result {
            Ok(res) => writeln!(cnsl, "{}", res.status()),
            Err(_) => writeln!(cnsl, "failed"),
        }
        .unwrap_or(());
        result
    }

    pub fn retry_send(&mut self) -> Result<Response> {
        let retry_interval = self.session.retry_interval().as_millis() as u64;
        let retry_limit = self.session.retry_limit();
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
    fn with_retry<'a>(
        self,
        client: &'a Client,
        session: &'a SessionConfig,
        cnsl: &'a mut Console,
    ) -> RetryRequestBuilder<'a>;
}

impl WithRetry for RequestBuilder {
    fn with_retry<'a>(
        self,
        client: &'a Client,
        session: &'a SessionConfig,
        cnsl: &'a mut Console,
    ) -> RetryRequestBuilder<'a> {
        RetryRequestBuilder {
            inner: self,
            client,
            session,
            cnsl,
        }
    }
}
