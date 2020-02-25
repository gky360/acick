use std::convert::TryFrom as _;
use std::fs::File;
use std::io::{BufReader, Seek as _, SeekFrom};

use anyhow::Context as _;
use cookie::Cookie as RawCookie;
use cookie_store::CookieStore;
use fs2::FileExt as _;
use reqwest::blocking::{Request, Response};
use reqwest::header::{HeaderValue, COOKIE, SET_COOKIE};

use crate::util::AbsPathBuf;
use crate::{Error, Result};

pub struct CookieStorage {
    file: File,
    store: CookieStore,
}

impl CookieStorage {
    pub fn open(path: &AbsPathBuf) -> Result<Self> {
        let file = path
            .create_dir_all_and_open(true, true)
            .context("Could not open cookies file")?;
        file.try_lock_exclusive()
            .context("Could not lock cookies file")?;
        let reader = BufReader::new(&file);
        let store = CookieStore::load_json(reader).map_err(Error::msg)?;
        Ok(Self { file, store })
    }

    pub fn load_into(&self, request: &mut Request) -> Result<()> {
        let url = request.url();
        let cookies = self
            .store
            .get_request_cookies(url)
            .map(|rc| rc.encoded().to_string());
        for cookie in cookies {
            request
                .headers_mut()
                .append(COOKIE, HeaderValue::try_from(cookie)?);
        }
        Ok(())
    }

    pub fn store_from(&mut self, response: &Response) -> Result<()> {
        let cookies = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|val| {
                val.to_str().ok().and_then(|cookie_str| {
                    match RawCookie::parse(cookie_str.to_owned()) {
                        Ok(raw_cookie) => Some(raw_cookie),
                        Err(_) => None,
                    }
                })
            });
        let url = response.url();
        self.store.store_response_cookies(cookies, url);
        self.save().context("Could not save cookies to json file")
    }

    pub fn save(&mut self) -> Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.set_len(0)?;
        self.store.save_json(&mut self.file).map_err(Error::msg)
    }
}

impl Drop for CookieStorage {
    fn drop(&mut self) {
        self.file.unlock().expect("Could no unlock cookies file");
    }
}
