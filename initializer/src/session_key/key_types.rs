use sc_cli::CryptoScheme;
pub use sp_application_crypto::key_types::{
    ACCOUNT, AURA, AUTHORITY_DISCOVERY, BABE, DUMMY, GRANDPA, IM_ONLINE, STAKING,
};
use sp_application_crypto::KeyTypeId;

pub const PARA_VALIDATOR: KeyTypeId = KeyTypeId(*b"para");
pub const PARA_ASSIGNMENT: KeyTypeId = KeyTypeId(*b"asgn");

pub trait KeyTypeIdExt {
    fn name(&self) -> Option<String>;

    fn crypto_scheme(&self) -> CryptoScheme;
}

impl KeyTypeIdExt for KeyTypeId {
    fn name(&self) -> Option<String> {
        let s = match *self {
            AURA => "aura",
            BABE => "babe",
            IM_ONLINE => "im_online",
            AUTHORITY_DISCOVERY => "authority_discovery",
            PARA_ASSIGNMENT => "para_assignment",
            PARA_VALIDATOR => "para_validator",
            GRANDPA => "grandpa",
            _ => return None,
        };

        Some(s.to_string())
    }

    fn crypto_scheme(&self) -> CryptoScheme {
        match *self {
            GRANDPA => CryptoScheme::Ed25519,
            AURA | BABE | IM_ONLINE | AUTHORITY_DISCOVERY | PARA_ASSIGNMENT | PARA_VALIDATOR => {
                CryptoScheme::Sr25519
            }
            _ => CryptoScheme::Sr25519,
        }
    }
}
