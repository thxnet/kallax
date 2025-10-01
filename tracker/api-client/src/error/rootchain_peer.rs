use core::fmt;

#[derive(Debug)]
pub enum GetRootchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },
}

impl From<kallax_primitives::Error> for GetRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

impl fmt::Display for GetRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum InsertRootchainPeerAddressError {
    Primitives { source: kallax_primitives::Error },
}

impl From<kallax_primitives::Error> for InsertRootchainPeerAddressError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

impl fmt::Display for InsertRootchainPeerAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
        }
    }
}
