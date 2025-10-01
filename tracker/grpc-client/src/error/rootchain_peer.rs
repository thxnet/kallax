use std::fmt;

#[derive(Debug)]
pub enum GetRootchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },

    Status { source: tonic::Status },
}

impl fmt::Display for GetRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Status { source } => source.fmt(f),
        }
    }
}

impl From<kallax_primitives::Error> for GetRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

#[derive(Debug)]
pub enum InsertRootchainPeerAddressError {
    Status { source: tonic::Status },
}

impl fmt::Display for InsertRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum ClearRootchainPeerAddressError {
    Status { source: tonic::Status },
}

impl fmt::Display for ClearRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}
