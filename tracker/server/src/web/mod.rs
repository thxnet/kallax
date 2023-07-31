pub mod controller;

pub mod extension {
    use crate::{ChainSpecList, PeerAddressBook};

    #[derive(Clone, Debug)]
    pub struct DomainName(pub String);

    #[derive(Clone, Debug)]
    pub struct RootchainPeerAddressBook(pub PeerAddressBook);

    #[derive(Clone, Debug)]
    pub struct LeafchainPeerAddressBook(pub PeerAddressBook);

    #[derive(Clone, Debug)]
    pub struct RootchainSpecList(pub ChainSpecList);

    #[derive(Clone, Debug)]
    pub struct LeafchainSpecList(pub ChainSpecList);
}
