mod leafchain_peer;
mod leafchain_spec;
mod rootchain_peer;
mod rootchain_spec;

use snafu::{Backtrace, Snafu};

pub use self::{
    leafchain_peer::{GetLeafchainPeerAddressError, InsertLeafchainPeerAddressError},
    leafchain_spec::GetLeafchainSpecError,
    rootchain_peer::{GetRootchainPeerAddressError, InsertRootchainPeerAddressError},
    rootchain_spec::GetRootchainSpecError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Error occurs while connecting to tracker endpoint `{endpoint}`, error: {source}"
    ))]
    ConnectToTrackerGrpc {
        endpoint: http::Uri,
        source: tonic::transport::Error,
        backtrace: Backtrace,
    },
}
