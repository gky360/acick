use lazy_static::lazy_static;
use reqwest::Url;

mod login;
mod settings;

pub use login::LoginPage;
pub use settings::SettingsPage;

lazy_static! {
    pub static ref BASE_URL: Url = Url::parse("https://atcoder.jp").unwrap();
}
