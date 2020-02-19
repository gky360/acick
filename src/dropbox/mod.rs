use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use anyhow::{anyhow, Context as _};
use dropbox_sdk::files::{
    self, FileMetadata, FolderMetadata, ListFolderArg, ListFolderContinueArg, ListFolderError,
    ListFolderResult, Metadata, PathROrId, SharedLink,
};
use dropbox_sdk::sharing::{self, GetSharedLinkFileArg, Path};
use tokio::task::{JoinError, JoinHandle};

use crate::{Error, Result};

mod authorizer;
mod hyper_client;

pub use authorizer::{DbxAuthorizer, Token};
use hyper_client::HyperClient;

pub static DBX_APP_KEY: &str = env!("ACICK_DBX_APP_KEY");
pub static DBX_APP_SECRET: &str = env!("ACICK_DBX_APP_SECRET");
pub static DBX_REDIRECT_PORT: u16 = 4100;
pub static DBX_REDIRECT_PATH: &str = "/oauth2/callback";

macro_rules! call_api {
    ($target:expr, $moved:expr) => {
        async {
            use tokio::time;

            static TIMEOUT_SECS: u64 = 1;
            time::timeout(
                std::time::Duration::from_millis(10),
                tokio::task::spawn_blocking(move || ($target, $moved)),
            )
            .await
            .context("Cancelled Dropbox API call due to timeout")
            .and_then(|res| res.context("API call panicked"))
            .and_then(|(res, moved)| Ok((res.map_err(convert_dbx_err)?, moved)))
            .and_then(|(res, moved)| Ok((res?, moved)))
        }
    };
}

struct Promise<T: Send> {
    handle: JoinHandle<T>,
}

impl<T: 'static + Send> Promise<T> {
    fn new<F: 'static + Send + FnOnce() -> T>(resolve: F) -> Self {
        let handle = tokio::task::spawn(async { resolve() });
        Promise { handle }
    }
}

impl<T: Send> Future for Promise<T> {
    type Output = std::result::Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        eprintln!("poll start");
        let ret = JoinHandle::poll(Pin::new(&mut self.handle), cx);
        eprintln!("poll end");
        ret
    }
}

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
}

fn list_folder(
    client: &HyperClient,
    arg: &ListFolderArg,
) -> dropbox_sdk::Result<std::result::Result<ListFolderResult, ListFolderError>> {
    eprintln!("start");
    let started = tokio::time::Instant::now();
    let res = files::list_folder(client, arg);
    eprintln!("received");
    for _ in 0..100_000_000 {}
    eprintln!("finished in :{}", started.elapsed().as_secs_f64());
    res
}

#[derive(Debug)]
pub struct Dropbox {
    client: Arc<HyperClient>,
}

impl Dropbox {
    pub fn new(token: Token) -> Self {
        let client = Arc::new(HyperClient::new(token.access_token));
        Self { client }
    }

    #[tokio::main]
    pub async fn list_all_folders<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<FolderMetadata>> {
        let folders = self
            .list_folder(path, shared_link_url)
            .await?
            .into_iter()
            .filter_map(|meta| match meta {
                Metadata::Folder(folder_meta) => Some(folder_meta),
                _ => None,
            })
            .collect();

        Ok(folders)
    }

    #[tokio::main]
    pub async fn list_all_files<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<FileMetadata>> {
        let files = self
            .list_folder(path, shared_link_url)
            .await?
            .into_iter()
            .filter_map(|meta| match meta {
                Metadata::File(file_meta) => Some(file_meta),
                _ => None,
            })
            .collect();

        Ok(files)
    }

    async fn list_folder<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<Metadata>> {
        let task = tokio::time::timeout(
            tokio::time::Duration::from_millis(10),
            Promise::new(move || {
                eprintln!("start");
                let started = tokio::time::Instant::now();
                for _ in 0..100_000_000 {}
                eprintln!("finished in {}", started.elapsed().as_secs_f64());
            }),
        );
        if task.await.is_err() {
            eprintln!("task timed out");
        }

        let mut arg = ListFolderArg::new(path.into());
        if let Some(shared_link_url) = shared_link_url {
            let shared_link = SharedLink::new(shared_link_url.to_owned());
            arg = arg.with_shared_link(Some(shared_link));
        }
        let client = self.client.clone();
        // let (mut res, client) = call_api!(list_folder(&client, &arg), client).await?;
        let mut res = tokio::time::timeout(
            std::time::Duration::from_millis(10),
            tokio::task::spawn_blocking(move || {
                eprintln!("start");
                let started = tokio::time::Instant::now();
                let res = files::list_folder(&*client, &arg);
                eprintln!("received");
                for _ in 0..100_000_000 {}
                eprintln!("finished in :{}", started.elapsed().as_secs_f64());
                res
            }),
        )
        .await
        .context("Cancelled Dropbox API call due to timeout")
        .and_then(|res| res.context("API call panicked"))
        .and_then(|res| res.map_err(convert_dbx_err))
        .and_then(|res| res.map_err(Into::into))?;
        let mut folders = res.entries;

        while res.has_more {
            let arg = ListFolderContinueArg { cursor: res.cursor };
            res = files::list_folder_continue(&*self.client, &arg).map_err(convert_dbx_err)??;
            folders.extend(res.entries.into_iter());
        }

        Ok(folders)
    }

    #[tokio::main]
    pub async fn get_shared_link_file<T: Into<String>>(
        &self,
        url: T,
        path: Path,
    ) -> Result<Box<dyn Read>> {
        let arg = GetSharedLinkFileArg::new(url.into()).with_path(Some(path.clone()));
        let res = sharing::get_shared_link_file(&*self.client, &arg, None, None)
            .map_err(convert_dbx_err)??;

        match res.body {
            Some(body) => Ok(body),
            _ => Err(anyhow!("Found empty body : {}", path)),
        }
    }
}
