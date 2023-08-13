mod leafchain_peer;
mod rootchain_peer;

use snafu::{Backtrace, Snafu};

pub use self::{
    leafchain_peer::GetLeafchainPeerAddressError, rootchain_peer::GetRootchainPeerAddressError,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Error occurs while connecting to tracker endpoint `{endpoint}`, error: {source}"
    ))]
    ConnectToTrackerApi { endpoint: http::Uri, source: reqwest::Error, backtrace: Backtrace },
}
