fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "proto/ExternalEndpoint.proto",
            "proto/LeafchainPeer.proto",
            "proto/LeafchainSpec.proto",
            "proto/PeerAddress.proto",
            "proto/RootchainPeer.proto",
            "proto/RootchainSpec.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
