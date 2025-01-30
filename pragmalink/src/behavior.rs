use libp2p::{
    StreamProtocol, identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    swarm::NetworkBehaviour,
};
use libp2p_gossipsub::MessageAuthenticity;
use std::time::Duration;

const PROTOCOL_VERSION: &str = "/pragma/kad/0.1.0";
const AGENT_VERSION: &str = "/pragma-node/0.1.0";

#[derive(NetworkBehaviour)]
pub struct P2pBehavior {
    pub gossipsub: libp2p_gossipsub::Behaviour,
    pub kademlia: libp2p::kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
}

impl P2pBehavior {
    pub fn new(local_keypair: Keypair, certificate: Option<String>) -> anyhow::Result<Self> {
        let local_peer_id = local_keypair.public().into();
        Ok(Self {
            identify: identify::Behaviour::new(
                identify::Config::new(identify::PROTOCOL_NAME.to_string(), local_keypair.public())
                    .with_agent_version(format!(
                        "{}/{}",
                        AGENT_VERSION,
                        certificate.unwrap_or("uncertified".to_string())
                    )),
            ),
            kademlia: {
                let protocol = StreamProtocol::try_from_owned(PROTOCOL_VERSION.into())
                    .expect("Invalid kad stream protocol");
                let mut cfg = kad::Config::new(protocol);
                cfg.set_periodic_bootstrap_interval(Some(Duration::from_millis(500)));
                kad::Behaviour::with_config(local_peer_id, MemoryStore::new(local_peer_id), cfg)
            },
            gossipsub: {
                let privacy = MessageAuthenticity::Signed(local_keypair.clone());
                libp2p_gossipsub::Behaviour::new(privacy, libp2p_gossipsub::Config::default())
                    .map_err(|err| anyhow::anyhow!("Error making gossipsub config: {err}"))?
            },
        })
    }
}
