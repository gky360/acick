use anyhow::Context as _;

use crate::{Error, Result};

pub fn open_in_browser(url: &str) -> Result<()> {
    match webbrowser::open(url) {
        Err(err) => Err(err.into()),
        Ok(output) if !output.status.success() => {
            Err(Error::msg("Process returned non-zero exit code"))
        }
        _ => Ok(()),
    }
    .with_context(|| format!("Could not open url in browser : {}", url))
}
