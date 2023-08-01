use std::{fmt, str::FromStr};

use snafu::OptionExt;

use crate::{error, error::Error};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExternalEndpoint {
    pub host: String,
    pub port: u16,
}

impl FromStr for ExternalEndpoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return error::InvalidEndpointSnafu { value: s.to_string() }.fail();
        }
        let host = parts[0].to_string();
        let port =
            parts[1].parse().ok().context(error::InvalidEndpointSnafu { value: s.to_string() })?;
        Ok(Self { host, port })
    }
}

impl fmt::Display for ExternalEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}
