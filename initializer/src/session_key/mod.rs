mod error;
pub mod key_types;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use sc_cli::CryptoScheme;
use snafu::ResultExt;
use sp_application_crypto::KeyTypeId;
use sp_core::crypto::{ExposeSecret, SecretString};

use self::error::Result;
pub use self::{error::Error, key_types::KeyTypeIdExt};

#[derive(Clone, Debug)]
pub struct SessionKey {
    phrase: SecretString,
    junctions: Vec<String>,
    key_type_id: KeyTypeId,
}

impl SessionKey {
    pub fn from_phrase<S: fmt::Display>(phrase: S, key_type_id: KeyTypeId) -> Self {
        Self::from_phrase_with_hard_junctions(phrase, Vec::new(), key_type_id)
    }

    pub fn from_phrase_with_hard_junctions<S: fmt::Display>(
        phrase: S,
        junctions: Vec<String>,
        key_type_id: KeyTypeId,
    ) -> Self {
        Self {
            phrase: SecretString::from(phrase.to_string().trim().to_string()),
            junctions,
            key_type_id,
        }
    }

    /// # Errors
    ///
    /// This function returns an error if the file is not saved.
    pub async fn save_file<P: AsRef<Path>>(&self, directory_path: P) -> Result<PathBuf> {
        fn extract_public_key<Pair: sp_core::Pair>(suri: &str) -> Result<Vec<u8>> {
            let pair: Pair = sp_core::Pair::from_string_with_seed(suri, None)
                .with_context(|_| error::GenerateKeyPairFromPhraseSnafu)?
                .0;
            Ok(pair.public().as_ref().to_vec())
        }

        let suri_str = format!(
            "{}//{}//{}",
            self.phrase.expose_secret().as_str(),
            self.junctions.iter().map(ToString::to_string).collect::<Vec<_>>().join("//"),
            self.key_type_id.name().expect("`name` must exist")
        );
        let file_name = {
            let public_key = {
                match self.key_type_id.crypto_scheme() {
                    CryptoScheme::Sr25519 => {
                        extract_public_key::<sp_core::sr25519::Pair>(&suri_str)?
                    }
                    CryptoScheme::Ed25519 => {
                        extract_public_key::<sp_core::ed25519::Pair>(&suri_str)?
                    }
                    CryptoScheme::Ecdsa => extract_public_key::<sp_core::ecdsa::Pair>(&suri_str)?,
                }
            };

            format!(
                "{}{}",
                array_bytes::bytes2hex("", self.key_type_id.0),
                array_bytes::bytes2hex("", public_key)
            )
        };

        let mut file_path = directory_path.as_ref().to_path_buf();
        file_path.push(file_name);

        tokio::fs::write(&file_path, serde_json::to_vec(&suri_str).expect("suri is valid JSON"))
            .await
            .with_context(|_| error::WriteFileSnafu { path: file_path.clone() })?;

        Ok(file_path)
    }
}
