use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum GetLeafchainPeerAddressError {
    #[snafu(context(suffix(G)))]
    Primitives { source: kallax_primitives::Error },

    #[snafu(context(suffix(G)), display("{source}"))]
    Reqwest { source: reqwest::Error },
    #[snafu(context(suffix(G)), display("{source}"))]
    UrlParse { source: url::ParseError },
    #[snafu(context(suffix(G)), display("UrlCanNotBeBase"))]
    UrlCanNotBeBase,
}

impl From<kallax_primitives::Error> for GetLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}

#[derive(Debug, Snafu)]
pub enum InsertLeafchainPeerAddressError {
    #[snafu(context(suffix(I)))]
    Primitives { source: kallax_primitives::Error },

    #[snafu(context(suffix(I)), display("{source}"))]
    Reqwest { source: reqwest::Error },
    #[snafu(context(suffix(I)), display("{source}"))]
    UrlParse { source: url::ParseError },
    #[snafu(context(suffix(I)), display("UrlCanNotBeBase"))]
    UrlCanNotBeBase,
}

impl From<kallax_primitives::Error> for InsertLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
