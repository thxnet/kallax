use std::{net::IpAddr, path::PathBuf};

use clap::Args;

use crate::consts;

#[derive(Args, Debug)]
pub struct Config {
    #[clap(long = "listen-address", default_value = "0.0.0.0", help = "Listen address")]
    pub listen_address: IpAddr,

    #[clap(long = "listen-port", default_value = consts::TRACKER_DEFAULT_PORT.to_string(), help = "Listen port")]
    pub listen_port: u16,

    #[clap(long = "rootchain-spec-files", help = "Rootchain spec files")]
    pub rootchain_spec_files: Vec<PathBuf>,

    #[clap(long = "leafchain-spec-files", help = "Leafchain spec files")]
    pub leafchain_spec_files: Vec<PathBuf>,
}
