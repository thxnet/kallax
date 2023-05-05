use std::fmt;

#[derive(Debug)]
pub enum GetLeafchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },

    Status { source: tonic::Status },
}

impl fmt::Display for GetLeafchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Status { source } => source.fmt(f),
        }
    }
}

impl From<kallax_primitives::Error> for GetLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}

#[derive(Debug)]
pub enum InsertLeafchainPeerAddressError {
    Status { source: tonic::Status },
}

impl fmt::Display for InsertLeafchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}
