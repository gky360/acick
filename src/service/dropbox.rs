use std::collections::HashMap;
use std::convert::Infallible;
use std::io::Read as _;
use std::net::SocketAddr;

use anyhow::Context as _;
use dropbox_sdk::{HyperClient, Oauth2AuthorizeUrlBuilder, Oauth2Type};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode, Uri};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng as _};
use tokio::sync::broadcast::{self, Sender};
use url::form_urlencoded;

use crate::abs_path::AbsPathBuf;
use crate::service::open_in_browser;
use crate::{Console, Error, Result};

static STATE_LEN: usize = 16;
static DBX_CODE_PARAM: &str = "code";
static DBX_STATE_PARAM: &str = "state";

pub struct Authorizer<'a> {
    app_key: &'a str,
    app_secret: &'a str,
    redirect_port: u16,
    redirect_path: &'a str,
    redirect_uri: String,
}

impl<'a> Authorizer<'a> {
    #[allow(dead_code)]
    pub fn new(
        app_key: &'a str,
        app_secret: &'a str,
        redirect_port: u16,
        redirect_path: &'a str,
    ) -> Self {
        Self {
            app_key,
            app_secret,
            redirect_port,
            redirect_path,
            redirect_uri: format!("http://localhost:{}{}", redirect_port, redirect_path),
        }
    }

    #[allow(dead_code)]
    pub fn load_or_request(&self, path: &AbsPathBuf, cnsl: &mut Console) -> Result<Dropbox> {
        let load_result = self
            .load_token(path, cnsl)
            .context("Could not load token")?;
        let token = match load_result {
            // TODO: check if token is valid
            Some(token) => token,
            None => self
                .request_token(cnsl)
                .context("Could not get access token from Dropbox")?,
        };

        let client = HyperClient::new(token);

        Ok(Dropbox { client })
    }

    fn load_token(&self, path: &AbsPathBuf, cnsl: &mut Console) -> Result<Option<String>> {
        if !path.as_ref().exists() {
            return Ok(None);
        }

        let mut token = String::new();
        path.load_pretty(
            |mut file| {
                file.read_to_string(&mut token)
                    .context("Could not load token from file")
            },
            None,
            cnsl,
        )?;

        Ok(Some(token))
    }

    fn request_token(&self, cnsl: &mut Console) -> Result<String> {
        let code = self.authorize(cnsl)?;
        HyperClient::oauth2_token_from_authorization_code(
            self.app_key,
            self.app_secret,
            &code,
            Some(&self.redirect_uri),
        )
        .map_err(convert_dbx_err)
    }

    #[tokio::main]
    async fn authorize(&self, cnsl: &mut Console) -> Result<String> {
        let (tx, mut rx) = broadcast::channel::<String>(1);

        // generate random state
        let state = gen_random_state();

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
        open_in_browser(auth_url.as_str(), cnsl)?;

        // wait for code to arrive and shutdown server
        let graceful = server.with_graceful_shutdown(async {
            let mut rx = tx.subscribe();
            rx.recv().await.unwrap();
            eprintln!("Shutting down server ...");
        });
        graceful.await?;

        Ok(rx.recv().await?)
    }
}

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
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
    eprintln!("{:?}", req);
    let res = if req.method() == Method::GET && req.uri().path() == redirect_path {
        handle_callback(req, tx, &state)
    } else {
        respond_not_found()
    };
    Ok(res)
}

pub struct Dropbox {
    client: HyperClient,
}
