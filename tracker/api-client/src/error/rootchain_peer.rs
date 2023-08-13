use std::fmt;

#[derive(Debug)]
pub enum GetRootchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },

    Error { source: reqwest::Error },
}

impl fmt::Display for GetRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Error { source } => source.fmt(f),
        }
    }
}

impl From<kallax_primitives::Error> for GetRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
