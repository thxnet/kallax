fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "proto/ChainMetadata.proto",
            "proto/ChainSpec.proto",
            "proto/PeerAddress.proto",
            "proto/Peer.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
