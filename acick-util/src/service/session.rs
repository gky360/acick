use std::io::Write as _;
use std::time::Duration;

use anyhow::Context as _;
use reqwest::blocking::{Client, Request, RequestBuilder, Response};
use retry::{delay, retry, OperationResult};

use crate::abs_path::AbsPathBuf;
use crate::service::CookieStorage;
use crate::{Console, Error, Result};

pub struct RetryRequestBuilder<'a> {
    inner: RequestBuilder,
    client: &'a Client,
    cookies_path: &'a AbsPathBuf,
    retry_limit: usize,
    retry_interval: Duration,
}

impl<'a> RetryRequestBuilder<'a> {
    pub fn retry_send(mut self, cnsl: &mut Console) -> Result<Response> {
        let retry_interval = self.retry_interval.as_millis() as u64;
        let durations = delay::Fixed::from_millis(retry_interval).take(self.retry_limit);
        retry(durations, || self.send(cnsl)).map_err(|err| match err {
            retry::Error::Operation { error, .. } => error,
            retry::Error::Internal(msg) => Error::msg(msg),
        })
    }

    fn send(&mut self, cnsl: &mut Console) -> OperationResult<Response, Error> {
        let result = self
            .inner
            .try_clone()
            .ok_or_else(|| Error::msg("Could not create request"))
            .and_then(|builder| Ok(builder.build()?))
            .context("Could not build request")
            .and_then(|req| self.exec_session_pretty(req, cnsl));
        match result {
            Ok(res) => {
                if res.status().is_server_error() {
                    OperationResult::Retry(Error::msg("Received server error"))
                } else {
                    OperationResult::Ok(res)
                }
            }
            Err(err) => OperationResult::Retry(err),
        }
    }

    fn exec_session_pretty(&mut self, req: Request, cnsl: &mut Console) -> Result<Response> {
        write!(cnsl, "{:7} {} ... ", req.method().as_str(), req.url()).unwrap_or(());
        let result = self.exec_session(req).context("Could not send request");
        match &result {
            Ok(res) => writeln!(cnsl, "{}", res.status()),
            Err(_) => writeln!(cnsl, "failed"),
        }
        .unwrap_or(());
        result
    }

    fn exec_session(&self, mut request: Request) -> Result<Response> {
        let mut storage =
            CookieStorage::open(self.cookies_path).context("Could not open cookie storage")?;
        storage
            .load_into(&mut request)
            .context("Could not load cookies into request")?;
        let response = self.client.execute(request)?;
        storage
            .store_from(&response)
            .context("Could not store cookies from response")?;
        Ok(response)
    }
}

pub trait WithRetry {
    fn with_retry<'a>(
        self,
        client: &'a Client,
        cookies_path: &'a AbsPathBuf,
        retry_limit: usize,
        retry_interval: Duration,
    ) -> RetryRequestBuilder<'a>;
}

impl WithRetry for RequestBuilder {
    fn with_retry<'a>(
        self,
        client: &'a Client,
        cookies_path: &'a AbsPathBuf,
        retry_limit: usize,
        retry_interval: Duration,
    ) -> RetryRequestBuilder<'a> {
        RetryRequestBuilder {
            inner: self,
            client,
            cookies_path,
            retry_limit,
            retry_interval,
        }
    }
}
