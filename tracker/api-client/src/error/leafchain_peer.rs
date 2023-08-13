use std::fmt;

#[derive(Debug)]
pub enum GetLeafchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },

    Error { source: reqwest::Error },
}

impl fmt::Display for GetLeafchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Error { source } => source.fmt(f),
        }
    }
}

impl From<kallax_primitives::Error> for GetLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}
