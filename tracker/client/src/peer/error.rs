use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    Primitive {
        source: kallax_tracker_primitives::Error,
    },

    #[snafu(display("Failed to get peer addresses: {source}"))]
    GetPeerAddresses {
        source: tonic::Status,
    },

    #[snafu(display("Failed to insert peer address: {source}"))]
    InsertPeerAddress {
        source: tonic::Status,
    },
}

impl From<kallax_tracker_primitives::Error> for Error {
    fn from(source: kallax_tracker_primitives::Error) -> Self { Self::Primitive { source } }
}
