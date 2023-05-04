pub mod chain_spec;
mod error;
pub mod peer;

use snafu::ResultExt;

pub use self::{
    chain_spec::ChainSpecExt,
    error::{Error, Result},
    peer::PeerExt,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub grpc_endpoint: http::Uri,
}

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
}

impl Client {
    pub async fn new(Config { grpc_endpoint }: Config) -> Result<Self> {
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .context(error::ConnectToTrackerGrpcSnafu)?;
        Ok(Self { channel })
    }
}
