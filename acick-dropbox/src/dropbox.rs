use std::fmt;
use std::io::Read;

use anyhow::anyhow;
use dropbox_sdk::default_client::UserAuthDefaultClient;
use dropbox_sdk::files::{
    self, FileMetadata, FolderMetadata, ListFolderArg, ListFolderContinueArg, ListFolderCursor,
    Metadata, PathROrId, SharedLink,
};
use dropbox_sdk::oauth2::Authorization;
use dropbox_sdk::sharing::{self, GetSharedLinkFileArg, Path};

use crate::convert_dbx_err;
use crate::Result;

pub struct Dropbox {
    client: UserAuthDefaultClient,
}

impl fmt::Debug for Dropbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dropbox")
            .field("client", &"client")
            .finish()
    }
}

impl Dropbox {
    pub fn new(auth: Authorization) -> Self {
        let client = UserAuthDefaultClient::new(auth);
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
