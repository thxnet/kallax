pub mod controller;

pub mod extension {
    use serde::Serialize;

    use crate::{ChainSpecList, PeerAddressBook};

    #[derive(Clone, Debug)]
    pub struct RootchainPeerAddressBook(pub PeerAddressBook);

    #[derive(Clone, Debug)]
    pub struct LeafchainPeerAddressBook(pub PeerAddressBook);

    #[derive(Clone, Debug)]
    pub struct RootchainSpecList(pub ChainSpecList);

    #[derive(Clone, Debug)]
    pub struct LeafchainSpecList(pub ChainSpecList);

    #[derive(Clone, Debug, Serialize)]
    pub struct TrackerConfig {
        pub peer_time_to_live_seconds: u64,
    }

    #[derive(Clone, Debug)]
    pub struct TrackerStartTime(pub std::time::Instant);
}
