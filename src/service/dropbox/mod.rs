use dropbox_sdk::files::{
    self, FolderMetadata, ListFolderArg, ListFolderContinueArg, ListFolderCursor, Metadata,
    SharedLink,
};
use dropbox_sdk::HyperClient;

use crate::{Error, Result};

mod authorizer;

pub use authorizer::DbxAuthorizer;

pub static DBX_APP_KEY: &str = env!("ACICK_DBX_APP_KEY");
pub static DBX_APP_SECRET: &str = env!("ACICK_DBX_APP_SECRET");
pub static DBX_REDIRECT_PORT: u16 = 4100;
pub static DBX_REDIRECT_PATH: &str = "/oauth2/callback";

fn convert_dbx_err(err: dropbox_sdk::Error) -> Error {
    Error::msg(err.to_string())
}

pub struct Dropbox {
    client: HyperClient,
}

impl Dropbox {
    pub fn list_all_folders(&self, shared_link_url: &str) -> Result<Vec<FolderMetadata>> {
        let root_path = String::from("");
        let shared_link = SharedLink::new(shared_link_url.to_owned());
        let arg = ListFolderArg::new(root_path).with_shared_link(Some(shared_link));
        let res = files::list_folder(&self.client, &arg).map_err(convert_dbx_err)??;

        let mut folders = res.entries;
        if res.has_more {
            self.list_folder_continue(res.cursor, &mut folders)?;
        }

        let folders = folders
            .into_iter()
            .filter_map(|meta| match meta {
                Metadata::Folder(folder_meta) => Some(folder_meta),
                _ => None,
            })
            .collect();

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
}
