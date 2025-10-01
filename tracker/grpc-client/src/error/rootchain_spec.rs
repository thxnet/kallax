use std::fmt;

#[derive(Debug)]
pub enum GetRootchainSpecError {
    Primitives { source: kallax_primitives::Error },

    Status { source: tonic::Status },

    MissingRootchainSpec,
}

impl From<kallax_primitives::Error> for GetRootchainSpecError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self {
        Self::Primitives { source }
    }
}

impl fmt::Display for GetRootchainSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Status { source } => source.fmt(f),
            Self::MissingRootchainSpec => f.write_str("Missing rootchain spec"),
        }
    }
}
