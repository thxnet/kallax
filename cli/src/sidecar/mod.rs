mod error;
mod options;

use std::{net::IpAddr, net::SocketAddr, time::Duration};

use kallax_primitives::ExternalEndpoint;
use kallax_sidecar::ChainEndpoint;

pub use self::{
    error::{Error, Result},
    options::Options,
};

const POLLING_INTERVAL: Duration = Duration::from_millis(1000);
const HETZNER_METADATA_URL: &str = "http://169.254.169.254/hetzner/v1/metadata/public-ipv4";
const FALLBACK_IP_DETECTION_URL: &str = "https://ifconfig.me/ip";
const PUBLIC_IP_DETECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// # Errors
///
/// This function returns an error if the sidecar is not created.
pub async fn run(options: Options) -> Result<()> {
    let config = {
        let Options {
            tracker_grpc_endpoint,
            rootchain_id,
            rootchain_node_websocket_endpoint,
            leafchain_id,
            leafchain_node_websocket_endpoint,
            allow_loopback_ip,
            external_rootchain_p2p_host,
            external_rootchain_p2p_port,
            external_leafchain_p2p_host,
            external_leafchain_p2p_port,
            auto_detect_public_ip,
            public_ip_detection_url,
            prefer_exposed_peers,
            diagnostic_listen_address,
            diagnostic_listen_port,
        } = options;

        let leafchain_endpoint = match (leafchain_id, leafchain_node_websocket_endpoint) {
            (Some(chain_id), Some(websocket_endpoint)) => {
                Some(ChainEndpoint { chain_id, websocket_endpoint })
            }
            (Some(_), None) => return Err(Error::LeafchainNodeWebSocketEndpointNotProvided),
            (None, Some(_)) => return Err(Error::LeafchainNameNotProvided),
            (None, None) => None,
        };

        let rootchain_endpoint = ChainEndpoint {
            chain_id: rootchain_id,
            websocket_endpoint: rootchain_node_websocket_endpoint,
        };

        // Auto-detect public IP if enabled
        let detected_ip = if auto_detect_public_ip {
            detect_public_ip(public_ip_detection_url.as_deref()).await
        } else {
            None
        };

        // Warn loudly if auto-detect was requested but failed and no explicit host set
        if auto_detect_public_ip && detected_ip.is_none() {
            if external_rootchain_p2p_host.is_none() || external_leafchain_p2p_host.is_none() {
                tracing::error!(
                    "Public IP auto-detection was enabled but failed, and no explicit \
                     --external-*-p2p-host was provided. P2P addresses may not be routable."
                );
            }
        }

        // Explicit --external-*-p2p-host takes priority over auto-detected IP
        let external_rootchain_p2p_endpoint =
            external_rootchain_p2p_host.or_else(|| detected_ip.clone()).map(|host| {
                ExternalEndpoint { host, port: external_rootchain_p2p_port.unwrap_or_default() }
            });
        let detected_public_ip = detected_ip.clone();
        let external_leafchain_p2p_endpoint =
            external_leafchain_p2p_host.or(detected_ip).map(|host| ExternalEndpoint {
                host,
                port: external_leafchain_p2p_port.unwrap_or_default(),
            });

        kallax_sidecar::Config {
            tracker_grpc_endpoint,
            polling_interval: POLLING_INTERVAL,
            rootchain_endpoint,
            leafchain_endpoint,
            allow_loopback_ip,
            prefer_exposed_peers,
            external_rootchain_p2p_endpoint,
            external_leafchain_p2p_endpoint,
            diagnostic_listen_address: SocketAddr::new(
                diagnostic_listen_address,
                diagnostic_listen_port,
            ),
            detected_public_ip,
        }
    };

    kallax_sidecar::serve(config).await?;

    Ok(())
}

async fn detect_public_ip(custom_url: Option<&str>) -> Option<String> {
    let client = reqwest::Client::builder().timeout(PUBLIC_IP_DETECTION_TIMEOUT).build().ok()?;

    let urls: Vec<&str> = if let Some(url) = custom_url {
        vec![url]
    } else {
        vec![HETZNER_METADATA_URL, FALLBACK_IP_DETECTION_URL]
    };

    for url in urls {
        tracing::info!("Attempting public IP detection via {url}");
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(ip) = resp.text().await {
                    let ip = ip.trim().to_string();
                    if ip.parse::<IpAddr>().is_ok() {
                        tracing::info!("Detected public IP: {ip} (via {url})");
                        return Some(ip);
                    }
                    tracing::warn!("Invalid IP from {url}: {ip}");
                }
            }
            Ok(resp) => tracing::warn!("Status {} from {url}", resp.status()),
            Err(err) => tracing::warn!("Failed to reach {url}: {err}"),
        }
    }
    tracing::warn!("Public IP auto-detection failed from all sources");
    None
}
