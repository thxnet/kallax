use serde::Deserialize;
use snafu::ResultExt;

use crate::{error, Error};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ChainSpec {
    id: String,
    body: Vec<u8>,
}

impl ChainSpec {
    #[must_use]
    pub fn id(&self) -> &str { self.id.as_str() }
}

impl TryFrom<&[u8]> for ChainSpec {
    type Error = Error;

    #[inline]
    fn try_from(body: &[u8]) -> Result<Self, Self::Error> {
        #[derive(Debug, Deserialize)]
        struct Model {
            id: String,
        }

        let json: Model = serde_json::from_slice(body).context(error::DeserializeChainSpecSnafu)?;

        if json.id.is_empty() {
            return Err(Error::MissingChainId);
        }

        Ok(Self { id: json.id, body: body.to_vec() })
    }
}

impl AsRef<[u8]> for ChainSpec {
    fn as_ref(&self) -> &[u8] { self.body.as_ref() }
}

#[cfg(test)]
mod tests {
    use crate::ChainSpec;

    #[test]
    fn test_try_from() {
        let chain_spec =
            ChainSpec::try_from(include_bytes!("test_data/chain_spec.json").as_ref()).unwrap();
        assert_eq!(chain_spec.id, "lmt_testnet");

        let chain_spec =
            ChainSpec::try_from(include_bytes!("test_data/chain_spec_no_id.json").as_ref());

        assert!(chain_spec.is_err());
    }
}
