use std::fs::{create_dir_all, File, OpenOptions};
use std::io::BufReader;
use std::path::Path;

use anyhow::Context as _;
use cookie_store::CookieStore;

use crate::{Error, Result};

pub struct CookieStorage {
    file: File,
    store: CookieStore,
}

impl CookieStorage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Some(dir) = path.as_ref().parent() {
            create_dir_all(dir).context("Could not create dir for cookies files")?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Could not open cookies file")?;
        let reader = BufReader::new(&file);
        let store = CookieStore::load_json(reader).map_err(Error::msg)?;
        Ok(Self { file, store })
    }
}
