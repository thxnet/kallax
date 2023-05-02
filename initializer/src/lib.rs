mod error;
pub mod node_key;
pub mod session_key;

use std::{
    fmt,
    path::{Path, PathBuf},
};

use kallax_tracker_client::{ChainSpecExt, Client as TrackerClient, Config as TrackerClientConfig};
use kallax_tracker_primitives::chain_spec::{ChainLayer, ChainMetadata};
use snafu::ResultExt;
use sp_application_crypto::KeyTypeId;
use sp_core::DeriveJunction;

pub use self::error::Error;
use self::{
    error::Result,
    node_key::NodeKey,
    session_key::{key_types, KeyTypeIdExt, SessionKey},
};

pub struct Config {
    node_key_file: PathBuf,

    tracker_grpc_endpoint: http::Uri,

    rootchain_name: String,
    rootchain_spec_file_path: PathBuf,

    leafchain_name: Option<String>,
    leafchain_spec_file_path: Option<PathBuf>,

    keystore_dir_path: PathBuf,
    session_key_mnemonic_phrase: String,
    node_name: String,
}

pub async fn prepare_session_keys<K, P, N>(
    keystore_dir_path: K,
    phrase: P,
    node_name: N,
) -> Result<()>
where
    K: AsRef<Path>,
    P: fmt::Display,
    N: fmt::Display,
{
    const SESSION_KEYS: &[KeyTypeId] = &[
        key_types::AURA,
        key_types::AUTHORITY_DISCOVERY,
        key_types::BABE,
        key_types::GRANDPA,
        key_types::IM_ONLINE,
        key_types::PARA_VALIDATOR,
        key_types::PARA_ASSIGNMENT,
    ];

    tokio::fs::create_dir_all(&keystore_dir_path).await.with_context(|_| {
        error::CreateDirectorySnafu { path: keystore_dir_path.as_ref().to_path_buf() }
    })?;

    for key_type_id in SESSION_KEYS {
        let session_key = SessionKey::from_phrase_with_junctions(
            &phrase,
            vec![DeriveJunction::hard(node_name.to_string())],
            *key_type_id,
        );

        let key_file_path = session_key.save_file(&keystore_dir_path).await?;
        tracing::info!(
            "Created session key {}, path: `{}`",
            key_type_id.name().expect("`name` must exist"),
            key_file_path.display()
        );
    }

    Ok(())
}

pub async fn prepare_chain_spec<S>(
    chain_metadata: ChainMetadata,
    chain_spec_file_path: S,
    tracker_client: &TrackerClient,
) -> Result<()>
where
    S: AsRef<Path>,
{
    let chain_spec =
        tracker_client.get(chain_metadata).await.with_context(|_| error::GetChainSpecSnafu)?;

    tokio::fs::write(&chain_spec_file_path, chain_spec).await.with_context(|_| {
        error::WriteFileSnafu { path: chain_spec_file_path.as_ref().to_path_buf() }
    })?;

    Ok(())
}

pub async fn execute(config: Config) -> Result<()> {
    let Config {
        node_key_file,
        keystore_dir_path,
        session_key_mnemonic_phrase,
        node_name,
        rootchain_name,
        rootchain_spec_file_path,
        leafchain_name,
        leafchain_spec_file_path,
        tracker_grpc_endpoint,
    } = config;

    // generate node key generate node key randomly and then save it
    let node_key = NodeKey::generate_random();
    node_key.save_file(node_key_file).await?;
    tracing::info!("Created node key with peer ID `{}`", node_key.peer_id());

    // generate session keys from mnemonic phrases or insert the existed keys
    prepare_session_keys(keystore_dir_path, session_key_mnemonic_phrase, node_name).await?;

    let tracker_client =
        TrackerClient::new(TrackerClientConfig { grpc_endpoint: tracker_grpc_endpoint }).await?;

    // fetch rootchain `chain_spec` from tracker and save it
    let rootchain_metadata = ChainMetadata { layer: ChainLayer::Rootchain, name: rootchain_name };
    prepare_chain_spec(rootchain_metadata, rootchain_spec_file_path, &tracker_client).await?;

    // fetch leafchain `chain_spec` from tracker and save it
    match (leafchain_name, leafchain_spec_file_path) {
        (Some(leafchain_name), Some(leafchain_spec_file_path)) => {
            let leafchain_metadata =
                ChainMetadata { layer: ChainLayer::Leafchain, name: leafchain_name };
            prepare_chain_spec(leafchain_metadata, leafchain_spec_file_path, &tracker_client)
                .await?;
        }
        _ => tracing::warn!(""),
    }

    // (optional) fetch bootnodes from tracker

    // start Substrate-based node
    // $EXECUTABLE \
    //    --no-hardware-benchmarks \
    //    --chain "$CHAIN_SPEC_PATH" \
    //    --name "$name" \
    //    --validator \
    //    --node-key-file "$node_key_file" \
    //    --base-path "$data_dir" \
    //    --keystore-path "$data_dir/keystore" \
    //    --port $((50001 + "${i}" * 10)) \
    //    --ws-port $((50002 + "${i}" * 10)) \
    //    --rpc-port $((50003 + "${i}" * 10)) \
    //    --rpc-cors all \
    //    --ws-external \
    //    --rpc-external \
    //    --rpc-methods unsafe \
    //    --allow-private-ip
    //    --discover-local
    //    --bootnodes "${BOOTNODES[@]}"
    //    --state-pruning=archive
    //    --blocks-pruning=archive
    //    --telemetry-url "wss://telemetry.testnet.thxnet.org/submit 0"

    Ok(())
}
