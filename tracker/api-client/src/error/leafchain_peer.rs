use core::fmt;

#[derive(Debug)]
pub enum GetLeafchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },
}

impl From<kallax_primitives::Error> for GetLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

impl fmt::Display for GetLeafchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum InsertLeafchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },
}

impl From<kallax_primitives::Error> for InsertLeafchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

impl fmt::Display for InsertLeafchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
        }
    }
}
