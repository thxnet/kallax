use std::borrow::Cow;

use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Unexpected data format: unknown value `{value}`",))]
    UnknownValue { value: Cow<'static, str>, backtrace: Backtrace },

    #[snafu(display("Invalid peer address `{value}`, error: {source}"))]
    InvalidPeerAddress { value: String, source: sc_network::multiaddr::Error },

    #[snafu(display("Failed to deserialize chain spec, error: {source}"))]
    DeserializeChainSpec { source: serde_json::Error },

    #[snafu(display("Could not parse chain ID"))]
    MissingChainId,
}
