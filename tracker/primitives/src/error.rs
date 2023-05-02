use std::borrow::Cow;

use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Unexpected data format: unknown value `{value}`",))]
    UnknownValue { value: Cow<'static, str>, backtrace: Backtrace },

    #[snafu(display("Invalid peer address `{value}`, error: {source}"))]
    InvalidPeerAddress { value: String, source: sc_network::multiaddr::Error },
}
