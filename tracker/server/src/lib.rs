mod chain_spec;
mod error;
mod peer;

use std::net::SocketAddr;

use kallax_tracker_proto::{chain_spec::ChainSpecServiceServer, peer::PeerServiceServer};
use snafu::ResultExt;

pub use self::error::{Error, Result};

#[derive(Clone, Debug)]
pub struct Config {
    listen_address: SocketAddr,
}

pub async fn start_grpc_server(Config { listen_address }: Config) -> Result<()> {
    tonic::transport::Server::builder()
        .add_service(ChainSpecServiceServer::new(chain_spec::Service::default()))
        .add_service(PeerServiceServer::new(peer::Service::default()))
        .serve(listen_address)
        .await
        .context(error::StartTonicServerSnafu)?;

    Ok(())
}
