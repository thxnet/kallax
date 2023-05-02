mod error;

use std::path::Path;

use libp2p::identity::{ed25519 as libp2p_ed25519, PublicKey};
use snafu::ResultExt;

pub use self::error::Error;
use self::error::Result;

#[derive(Clone, Debug)]
pub struct NodeKey {
    keypair: libp2p_ed25519::Keypair,
}

impl NodeKey {
    #[inline]
    pub fn generate_random() -> Self { Self { keypair: libp2p_ed25519::Keypair::generate() } }

    #[inline]
    pub async fn save_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let secret = self.keypair.secret();
        tokio::fs::write(&path, secret)
            .await
            .with_context(|_| error::WriteFileSnafu { path: path.as_ref().to_path_buf() })?;
        Ok(())
    }

    #[inline]
    pub fn peer_id(&self) -> String {
        PublicKey::Ed25519(self.keypair.public()).to_peer_id().to_string()
    }
}
