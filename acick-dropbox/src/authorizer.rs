use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Write;
use std::net::SocketAddr;

use anyhow::Context as _;
use dropbox_sdk::check::{self, EchoArg};
use dropbox_sdk::ErrorKind;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode, Uri};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng as _};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{self, Sender};
use url::form_urlencoded;

use crate::abs_path::AbsPathBuf;
use crate::hyper_client::{HyperClient, Oauth2AuthorizeUrlBuilder, Oauth2Type};
use crate::web::open_in_browser;
use crate::Result;
use crate::{convert_dbx_err, Dropbox};

static STATE_LEN: usize = 16;
static DBX_CODE_PARAM: &str = "code";
static DBX_STATE_PARAM: &str = "state";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub access_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DbxAuthorizer<'a> {
    app_key: &'a str,
    app_secret: &'a str,
    redirect_port: u16,
    redirect_path: &'a str,
    redirect_uri: String,
    token_path: &'a AbsPathBuf,
    access_token: Option<String>,
}

impl<'a> DbxAuthorizer<'a> {
    pub fn new(
        app_key: &'a str,
        app_secret: &'a str,
        redirect_port: u16,
        redirect_path: &'a str,
        token_path: &'a AbsPathBuf,
        access_token: Option<String>,
    ) -> Self {
        Self {
            app_key,
            app_secret,
            redirect_port,
            redirect_path,
            redirect_uri: format!("http://localhost:{}{}", redirect_port, redirect_path),
            token_path,
            access_token,
        }
    }

    pub fn load_or_request(&self, cnsl: &mut dyn Write) -> Result<Dropbox> {
        let load_result = self.load_token(cnsl)?;
        let (token, is_updated) = match load_result {
            Some(token) if Self::validate_token(&token)? => (token, false),
            _ => (self.request_token(cnsl)?, true),
        };

        if is_updated {
            self.save_token(&token, cnsl)?;
        }

        Ok(Dropbox::new(token))
    }

    fn load_token(&self, cnsl: &mut dyn Write) -> Result<Option<Token>> {
        if let Some(access_token) = &self.access_token {
            return Ok(Some(Token {
                access_token: access_token.to_owned(),
            }));
        }

        if !self.token_path.as_ref().exists() {
            return Ok(None);
        }

        let token = self.token_path.load_pretty(
            |file| serde_json::from_reader(file).context("Could not load token from json file"),
            None,
            cnsl,
        )?;

        Ok(Some(token))
    }

    fn save_token(&self, token: &Token, cnsl: &mut dyn Write) -> Result<()> {
        self.token_path.save_pretty(
            |file| serde_json::to_writer(file, token).context("Could not save token as json file"),
            true,
            None,
            cnsl,
        )?;

        Ok(())
    }

    fn validate_token(token: &Token) -> Result<bool> {
        let client = HyperClient::new(token.access_token.clone());
        match check::user(&client, &EchoArg { query: "".into() }) {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(())) => Ok(false),
            Err(dropbox_sdk::Error(ErrorKind::InvalidToken(_), ..)) => Ok(false),
            Err(err) => Err(convert_dbx_err(err)),
        }
        .context("Could not validate access token")
    }

    #[tokio::main]
    async fn request_token(&self, cnsl: &mut dyn Write) -> Result<Token> {
        let state = gen_random_state();
        let code = self
            .authorize(state, cnsl)
            .await
            .context("Could not authorize acick on Dropbox")?;
        let access_token = HyperClient::oauth2_token_from_authorization_code(
            self.app_key,
            self.app_secret,
            &code,
            Some(&self.redirect_uri),
        )
        .map_err(convert_dbx_err)
        .context("Could not get access token from Dropbox")?;

        Ok(Token { access_token })
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
        let auth_url = Oauth2AuthorizeUrlBuilder::new(self.app_key, Oauth2Type::AuthorizationCode)
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

    #[tokio::test]
    async fn test_authorize() -> anyhow::Result<()> {
        let token_path = AbsPathBuf::try_new("/tmp/dbx_token.json")?;
        let authorizer =
            DbxAuthorizer::new("test_key", "test_secret", 4100, "/path", &token_path, None);
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
