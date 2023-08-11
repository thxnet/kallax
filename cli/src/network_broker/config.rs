use clap::Args;

use crate::network_broker::{CONFIG_PATH, TRACKER_API_ENDPOINT};

#[derive(Args, Debug)]
pub struct Config {
    #[clap(long = "tracker-api-endpoint", help = "Tracker api endpoint", default_value = TRACKER_API_ENDPOINT)]
    pub tracker_api_endpoint: http::Uri,

    #[clap(long = "file", help = "Config file path", default_value = CONFIG_PATH)]
    pub file: String,
}
