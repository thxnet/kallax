use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum GetLeafchainPeerAddressError {
    Primitives {
        source: kallax_primitives::Error,
    },

    #[snafu(display("{source}"))]
    Reqwest {
        source: reqwest::Error,
    },
    #[snafu(display("{source}"))]
    UrlParse {
        source: url::ParseError,
    },
    #[snafu(display("UrlCanNotBeBase"))]
    UrlCanNotBeBase,
}

impl From<kallax_primitives::Error> for GetLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
