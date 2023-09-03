use std::path::PathBuf;

use clap::Args;

use crate::network_broker::{CONFIG_PATH, TRACKER_API_ENDPOINT};

#[derive(Args, Debug)]
pub struct Options {
    #[clap(long = "tracker-api-endpoint", help = "Tracker api endpoint", default_value = TRACKER_API_ENDPOINT)]
    pub tracker_api_endpoint: http::Uri,

    #[clap(short = 'f', long = "file", help = "Config file path", default_value = CONFIG_PATH)]
    pub file: PathBuf,
}
