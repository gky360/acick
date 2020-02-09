use dropbox_sdk::HyperClient;

mod authorizer;

pub struct Dropbox {
    client: HyperClient,
}
