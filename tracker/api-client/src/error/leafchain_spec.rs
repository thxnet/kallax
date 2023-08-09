use core::fmt;

#[derive(Debug)]
pub enum GetLeafchainSpecError {
    Primitives { source: kallax_primitives::Error },

    Status { source: tonic::Status },

    MissingLeafchainSpec,
}

impl From<kallax_primitives::Error> for GetLeafchainSpecError {
    #[inline]
    fn from(source: kallax_primitives::Error) -> Self { Self::Primitives { source } }
}

impl fmt::Display for GetLeafchainSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitives { source } => source.fmt(f),
            Self::Status { source } => source.fmt(f),
            Self::MissingLeafchainSpec => f.write_str("Missing leafchain spec"),
        }
    }
}
