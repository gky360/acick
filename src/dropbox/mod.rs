use std::io::Read;

use anyhow::anyhow;
use dropbox_sdk::files::{
    self, FileMetadata, FolderMetadata, ListFolderArg, ListFolderContinueArg, ListFolderCursor,
    Metadata, PathROrId, SharedLink,
};
use dropbox_sdk::sharing::{self, GetSharedLinkFileArg, Path};

use crate::{Error, Result};

mod authorizer;
mod hyper_client;

pub use authorizer::{DbxAuthorizer, Token};
use hyper_client::HyperClient;

pub static DBX_APP_KEY: &str = env!("ACICK_DBX_APP_KEY");
pub static DBX_APP_SECRET: &str = env!("ACICK_DBX_APP_SECRET");
pub static DBX_REDIRECT_PORT: u16 = 4100;
pub static DBX_REDIRECT_PATH: &str = "/oauth2/callback";

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
}

#[derive(Debug)]
pub struct Dropbox {
    client: HyperClient,
}

impl Dropbox {
    pub fn new(token: Token) -> Self {
        let client = HyperClient::new(token.access_token);
        Self { client }
    }

    pub fn list_all_folders<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<FolderMetadata>> {
        let folders = self
            .list_folder(path, shared_link_url)?
            .into_iter()
            .filter_map(|meta| match meta {
                Metadata::Folder(folder_meta) => Some(folder_meta),
                _ => None,
            })
            .collect();

        Ok(folders)
    }

    pub fn list_all_files<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<FileMetadata>> {
        let files = self
            .list_folder(path, shared_link_url)?
            .into_iter()
            .filter_map(|meta| match meta {
                Metadata::File(file_meta) => Some(file_meta),
                _ => None,
            })
            .collect();

        Ok(files)
    }

    fn list_folder<P: Into<PathROrId>>(
        &self,
        path: P,
        shared_link_url: Option<&str>,
    ) -> Result<Vec<Metadata>> {
        let mut arg = ListFolderArg::new(path.into());
        if let Some(shared_link_url) = shared_link_url {
            let shared_link = SharedLink::new(shared_link_url.to_owned());
            arg = arg.with_shared_link(Some(shared_link));
        }
        let res = files::list_folder(&self.client, &arg).map_err(convert_dbx_err)??;

        let mut folders = res.entries;
        if res.has_more {
            self.list_folder_continue(res.cursor, &mut folders)?;
        }

        Ok(folders)
    }

    fn list_folder_continue(
        &self,
        cursor: ListFolderCursor,
        folders: &mut Vec<Metadata>,
    ) -> Result<()> {
        let arg = ListFolderContinueArg { cursor };
        let res = files::list_folder_continue(&self.client, &arg).map_err(convert_dbx_err)??;
        folders.extend(res.entries.into_iter());
        if res.has_more {
            self.list_folder_continue(res.cursor, folders)?;
        }
        Ok(())
    }

    pub fn get_shared_link_file<T: Into<String>>(
        &self,
        url: T,
        path: Path,
    ) -> Result<Box<dyn Read>> {
        let arg = GetSharedLinkFileArg::new(url.into()).with_path(Some(path.clone()));
        let res = sharing::get_shared_link_file(&self.client, &arg, None, None)
            .map_err(convert_dbx_err)??;

        match res.body {
            Some(body) => Ok(body),
            _ => Err(anyhow!("Found empty body : {}", path)),
        }
    }
}
