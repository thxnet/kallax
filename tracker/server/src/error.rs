use std::fmt;

use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while starting tonic server, error: {source}"))]
    StartTonicServer { source: tonic::transport::Error, backtrace: Backtrace },
}

#[must_use]
pub fn into_invalid_argument_status(err: impl fmt::Display) -> tonic::Status {
    tonic::Status::invalid_argument(err.to_string())
}
