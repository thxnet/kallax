use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum GetRootchainPeerAddressError {
    #[snafu(context(suffix(G)))]
    Primitives { source: kallax_primitives::Error },

    #[snafu(context(suffix(G)), display("{source}"))]
    Reqwest { source: reqwest::Error },
    #[snafu(context(suffix(G)), display("{source}"))]
    UrlParse { source: url::ParseError },
    #[snafu(context(suffix(G)), display("UrlCanNotBeBase"))]
    UrlCanNotBeBase,
}

impl From<kallax_primitives::Error> for GetRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}

#[derive(Debug, Snafu)]
pub enum InsertRootchainPeerAddressError {
    #[snafu(context(suffix(I)))]
    Primitives { source: kallax_primitives::Error },

    #[snafu(context(suffix(I)), display("{source}"))]
    Reqwest { source: reqwest::Error },
    #[snafu(context(suffix(I)), display("{source}"))]
    UrlParse { source: url::ParseError },
    #[snafu(context(suffix(I)), display("UrlCanNotBeBase"))]
    UrlCanNotBeBase,
}

impl From<kallax_primitives::Error> for InsertRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
