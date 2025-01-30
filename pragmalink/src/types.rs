use libp2p::identify;

pub enum P2pRequest {
    Broadcast(String, Vec<u8>),
}

pub struct ReceivedConnection {
    pub peer_id: String,
    pub pubkey: [u8; 32],
    pub listen_addrs: Vec<String>,
    pub observed_addr: String,
    pub certificate: Option<String>,
}

impl TryFrom<identify::Event> for ReceivedConnection {
    type Error = anyhow::Error;
    fn try_from(event: identify::Event) -> anyhow::Result<Self> {
        match event {
            identify::Event::Received { peer_id, info, .. } => {
                let pubkey = info.public_key.try_into_ed25519()?.to_bytes();
                let listen_addrs = info
                    .listen_addrs
                    .iter()
                    .map(|addr| addr.to_string())
                    .collect();
                let observed_addr = info.observed_addr.to_string();
                let certificate = extract_certificate_from_agent_version(&info.agent_version);
                Ok(ReceivedConnection {
                    peer_id: peer_id.to_string(),
                    pubkey,
                    listen_addrs,
                    certificate,
                    observed_addr,
                })
            }
            _ => {
                tracing::debug!("invalid identify event for a received connection");
                Err(anyhow::anyhow!("invalid identify event"))
            }
        }
    }
}

fn extract_certificate_from_agent_version(agent_version: &str) -> Option<String> {
    agent_version.split('/').nth(3).map(|s| s.into())
}
