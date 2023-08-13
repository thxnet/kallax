use std::fmt;

#[derive(Debug)]
pub enum GetRootchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },

    Reqwest { source: reqwest::Error },
    UrlParse { source: url::ParseError },
    UrlCanNotBeBase { source: () },
}

impl fmt::Display for GetRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Reqwest { source } => source.fmt(f),
            Self::UrlParse { source } => source.fmt(f),
            Self::UrlCanNotBeBase { source: _ } => "()".fmt(f),
        }
    }
}

impl From<kallax_primitives::Error> for GetRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
