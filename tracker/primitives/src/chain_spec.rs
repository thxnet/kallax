use kallax_tracker_proto::chain_spec as proto;
use snafu::OptionExt;

use crate::{error, error::Error};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChainLayer {
    Rootchain,
    Leafchain,
}

impl TryFrom<proto::ChainLayer> for ChainLayer {
    type Error = Error;

    fn try_from(value: proto::ChainLayer) -> Result<Self, Self::Error> {
        match value {
            proto::ChainLayer::Rootchain => Ok(Self::Rootchain),
            proto::ChainLayer::Leafchain => Ok(Self::Leafchain),
        }
    }
}

impl From<ChainLayer> for proto::ChainLayer {
    fn from(value: ChainLayer) -> Self {
        match value {
            ChainLayer::Rootchain => Self::Rootchain,
            ChainLayer::Leafchain => Self::Leafchain,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ChainMetadata {
    pub layer: ChainLayer,
    pub name: String,
}

impl TryFrom<proto::ChainMetadata> for ChainMetadata {
    type Error = Error;

    fn try_from(
        proto::ChainMetadata { layer, name }: proto::ChainMetadata,
    ) -> Result<Self, Self::Error> {
        let layer = {
            let layer = proto::ChainLayer::from_i32(layer)
                .with_context(|| error::UnknownValueSnafu { value: layer.to_string() })?;

            ChainLayer::try_from(layer)?
        };

        Ok(ChainMetadata { layer, name })
    }
}

impl From<ChainMetadata> for proto::ChainMetadata {
    fn from(ChainMetadata { layer, name }: ChainMetadata) -> Self {
        let layer = proto::ChainLayer::from(layer).into();
        proto::ChainMetadata { layer, name }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ChainSpec(Vec<u8>);

impl TryFrom<proto::ChainSpec> for ChainSpec {
    type Error = Error;

    fn try_from(proto::ChainSpec { data }: proto::ChainSpec) -> Result<Self, Self::Error> {
        Ok(Self(data))
    }
}

impl From<ChainSpec> for proto::ChainSpec {
    fn from(ChainSpec(data): ChainSpec) -> Self { Self { data } }
}
