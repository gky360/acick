use std::collections::HashMap;
use std::convert::Infallible;
use std::io::{Read, Write};
use std::net::SocketAddr;

use anyhow::Context as _;
use dropbox_sdk::default_client::NoauthDefaultClient;
use dropbox_sdk::oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode, Uri};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng as _};
use tokio::sync::broadcast::{self, Sender};
use url::form_urlencoded;

use crate::abs_path::AbsPathBuf;
use crate::web::open_in_browser;
use crate::{Dropbox, Result};

static STATE_LEN: usize = 16;
static DBX_CODE_PARAM: &str = "code";
static DBX_STATE_PARAM: &str = "state";

#[derive(Debug, Clone)]
pub struct DbxAuthorizer<'a> {
    client_id: &'a str,
    redirect_port: u16,
    redirect_path: &'a str,
    redirect_uri: String,
    token_path: &'a AbsPathBuf,
    oauth2_flow: Oauth2Type,
}

impl<'a> DbxAuthorizer<'a> {
    pub fn new(
        client_id: &'a str,
        redirect_port: u16,
        redirect_path: &'a str,
        token_path: &'a AbsPathBuf,
    ) -> Self {
        Self {
            client_id,
            redirect_port,
            redirect_path,
            redirect_uri: format!("http://localhost:{}{}", redirect_port, redirect_path),
            token_path,
            oauth2_flow: Oauth2Type::PKCE(PkceCode::new()),
        }
    }

    pub fn load_or_request(
        &self,
        access_token: Option<String>,
        cnsl: &mut dyn Write,
    ) -> Result<Dropbox> {
        let load_result = self.load_token(access_token, cnsl)?;
        let (mut auth, is_updated) = match load_result {
            Some(auth) => (auth, false),
            _ => (self.request_token(cnsl)?, true),
        };

        let client = NoauthDefaultClient::default();
        auth.obtain_access_token(client)
            .context("Failed to obtain dropbox access token")?;

        if is_updated {
            self.save_token(&auth, cnsl)?;
        }

        Ok(Dropbox::new(auth))
    }

    fn load_token(
        &self,
        access_token: Option<String>,
        cnsl: &mut dyn Write,
    ) -> Result<Option<Authorization>> {
        if let Some(access_token) = access_token {
            return Ok(Some(Authorization::from_access_token(access_token)));
        }

        if !self.token_path.as_ref().exists() {
            return Ok(None);
        }

        let auth = self.token_path.load_pretty(
            |mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf)
                    .context("Could not load token from file")?;
                Ok(Authorization::load(self.client_id.to_string(), &buf))
            },
            None,
            cnsl,
        )?;

        Ok(auth)
    }

    fn save_token(&self, auth: &Authorization, cnsl: &mut dyn Write) -> Result<()> {
        self.token_path.save_pretty(
            |mut file| {
                file.write_all(auth.save().unwrap_or_default().as_bytes())
                    .context("Could not save token as file")
            },
            true,
            None,
            cnsl,
        )?;

        Ok(())
    }

    #[tokio::main]
    async fn request_token(&self, cnsl: &mut dyn Write) -> Result<Authorization> {
        let state = gen_random_state();
        let auth_code = self
            .authorize(state, cnsl)
            .await
            .context("Could not authorize acick on Dropbox")?;

        let auth = Authorization::from_auth_code(
            self.client_id.to_string(),
            self.oauth2_flow.clone(),
            auth_code.trim().to_owned(),
            Some(self.redirect_uri.to_owned()),
        );

        Ok(auth)
    }

    async fn authorize(&self, state: String, cnsl: &mut dyn Write) -> Result<String> {
        let (tx, mut rx) = broadcast::channel::<String>(1);

        // start local server
        let addr = SocketAddr::from(([127, 0, 0, 1], self.redirect_port));
        let make_service = make_service_fn(|_conn| {
            let redirect_path = self.redirect_path.to_owned();
            let state = state.clone();
            let tx = tx.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |req| {
                    respond(req, redirect_path.clone(), state.clone(), tx.clone())
                }))
            }
        });
        let server = Server::bind(&addr).serve(make_service);

        // open auth url in browser
        let auth_url = AuthorizeUrlBuilder::new(self.client_id, &self.oauth2_flow)
            .redirect_uri(&self.redirect_uri)
            .state(&state)
            .build();
        open_in_browser(auth_url.as_str())
            .context("Could not open a url in browser")
            // coerce error
            .unwrap_or_else(|err| writeln!(cnsl, "{}", err).unwrap_or(()));
        writeln!(cnsl, "Authorize Dropbox in web browser.")?;

        // wait for code to arrive and shutdown server
        let graceful = server.with_graceful_shutdown(async {
            let mut rx = tx.subscribe();
            rx.recv().await.unwrap();
        });
        graceful.await?;

        Ok(rx.recv().await?)
    }
}

fn gen_random_state() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(STATE_LEN)
        .collect()
}

fn get_params(uri: &Uri) -> HashMap<String, String> {
    uri.query()
        .map(|query_str| {
            form_urlencoded::parse(query_str.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new)
}

fn respond_param_missing(name: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(format!("Missing parameter: {}", name)))
        .unwrap()
}

fn respond_param_invalid(name: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from(format!("Invalid parameter: {}", name)))
        .unwrap()
}

fn respond_not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

fn handle_callback(req: Request<Body>, tx: Sender<String>, state_expected: &str) -> Response<Body> {
    let mut params = get_params(req.uri());
    let code = match params.remove(DBX_CODE_PARAM) {
        Some(code) => code,
        None => return respond_param_missing(DBX_CODE_PARAM),
    };
    let state = match params.remove(DBX_STATE_PARAM) {
        Some(state) => state,
        None => return respond_param_missing(DBX_STATE_PARAM),
    };
    if state != state_expected {
        return respond_param_invalid(DBX_STATE_PARAM);
    }

    // send auth code to Authorizer
    tx.send(code).unwrap_or(0);

    Response::new(Body::from(
        "Successfully completed authorization. Go back to acick on your terminal.",
    ))
}

async fn respond(
    req: Request<Body>,
    redirect_path: String,
    state: String,
    tx: Sender<String>,
) -> std::result::Result<Response<Body>, Infallible> {
    if req.method() == Method::GET && req.uri().path() == redirect_path {
        return Ok(handle_callback(req, tx, &state));
    }
    Ok(respond_not_found())
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};

    use super::*;

    macro_rules! map(
        { $($key:expr => $value:expr),+ } => {
            {
                let mut m = ::std::collections::HashMap::new();
                $(
                    m.insert($key, $value);
                )+
                m
            }
         };
    );

    fn run_test(f: fn(test_dir: &TempDir, authorizer: DbxAuthorizer) -> anyhow::Result<()>) {
        let test_dir = tempdir().unwrap();
        let token_path = AbsPathBuf::try_new(test_dir.path().join("dbx_token.txt")).unwrap();
        let authorizer = DbxAuthorizer::new("test_id", 4100, "/path", &token_path);
        f(&test_dir, authorizer).unwrap();
    }

    #[test]
    fn test_load_token() {
        run_test(|_, authorizer| {
            let access_token = "test_token".to_string();
            let auth = Authorization::from_access_token(access_token.to_owned());
            let mut buf = Vec::new();

            let actual = authorizer
                .load_token(Some(access_token), &mut buf)?
                .and_then(|auth| auth.save());
            let expected = auth.save();
            assert_eq!(actual, expected);

            assert!(authorizer.load_token(None, &mut buf)?.is_none());

            let token_path = authorizer.token_path.as_ref();
            let mut file = std::fs::File::create(token_path)?;
            file.write_all(b"1&test_token")?;

            let actual = authorizer
                .load_token(None, &mut buf)?
                .and_then(|auth| auth.save());
            assert_eq!(actual, expected);

            Ok(())
        })
    }

    #[test]
    fn test_save_token() {
        run_test(|_, authorizer| {
            let access_token = "test_token".to_string();
            let auth = Authorization::from_access_token(access_token);
            let mut buf = Vec::<u8>::new();
            authorizer.save_token(&auth, &mut buf)?;
            let token_str = std::fs::read_to_string(authorizer.token_path.as_ref())?;
            assert_eq!(token_str, "1&test_token");
            Ok(())
        })
    }

    #[tokio::test]
    async fn test_authorize() -> anyhow::Result<()> {
        let test_dir = tempdir().unwrap();
        let token_path = AbsPathBuf::try_new(test_dir.path().join("dbx_token.txt")).unwrap();
        let authorizer = DbxAuthorizer::new("test_id", 4100, "/path", &token_path);
        let mut buf = Vec::<u8>::new();
        let future = authorizer.authorize("test_state".to_string(), &mut buf);

        tokio::spawn(async {
            let client = hyper::Client::new();
            let uri =
                Uri::from_static("http://localhost:4100/path?code=test_code&state=test_state");
            client.get(uri).await.unwrap();
        });

        let code = future.await?;
        assert_eq!(code, "test_code");
        Ok(())
    }

    #[test]
    fn test_gen_random_state() {
        assert_eq!(gen_random_state().len(), STATE_LEN);
        assert_ne!(gen_random_state(), gen_random_state());
    }

    #[test]
    fn test_get_params() {
        let tests = &[
            (Uri::from_static("http://example.com/"), HashMap::new()),
            (Uri::from_static("http://example.com/?"), HashMap::new()),
            (
                Uri::from_static("http://example.com/?hoge=fuga&foo=bar"),
                map!(String::from("hoge") => String::from("fuga"), String::from("foo") => String::from("bar")),
            ),
        ];

        for (left, expected) in tests {
            let actual = get_params(left);
            assert_eq!(&actual, expected);
        }
    }

    #[tokio::test]
    async fn test_respond() -> anyhow::Result<()> {
        let tests = &[
            ("/path?code=test_code&state=test_state", StatusCode::OK),
            ("/path", StatusCode::BAD_REQUEST),
            ("/path?code=test_code", StatusCode::BAD_REQUEST),
            (
                "/path?code=test_code&state=invalid_state",
                StatusCode::BAD_REQUEST,
            ),
            (
                "/invalid_path?code=test_code&state=test_state",
                StatusCode::NOT_FOUND,
            ),
        ];

        for (left, expected) in tests {
            let (tx, mut rx) = broadcast::channel::<String>(2);
            let req = Request::get(format!("http://localhost:4100{}", left)).body(Body::empty())?;
            let redirect_path = "/path".to_string();
            let state = "test_state".to_string();
            let res = respond(req, redirect_path, state, tx).await?;
            assert_eq!(res.status(), *expected);
            if res.status() == StatusCode::OK {
                let code = rx.recv().await?;
                assert_eq!(code, "test_code");
            }
        }
        Ok(())
    }
}
