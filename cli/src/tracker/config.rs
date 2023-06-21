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

    #[clap(
        long = "allow-peer-in-loopback-network",
        help = "Allow to track peers in loopback network"
    )]
    pub allow_peer_in_loopback_network: bool,

    #[clap(
        long = "peer-time-to-live",
        default_value = consts::TRACKER_DEFAULT_PEER_TIME_TO_LIVE_SECONDS.to_string(),
        help = "Time-to-live of Peer in seconds"
    )]
    pub peer_time_to_live: u64,
}
