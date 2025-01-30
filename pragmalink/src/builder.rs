use crate::{DEFAULT_LISTENING_PORT, P2pNode, types::P2pRequest};
use auth_rs::AuthorityCertificate;
use libp2p::{Multiaddr, identity::Keypair, multiaddr::Protocol};
use libp2p_gossipsub::Message;
use std::collections::HashSet;

pub struct P2pNodeBuilder {
    keypair: Option<Keypair>,
    listening_address: Option<Multiaddr>,
    bootstrap_nodes: Option<HashSet<Multiaddr>>,
    indentify_certificate: Option<AuthorityCertificate>,
    gossipsub_topics: Option<HashSet<String>>,
}

impl P2pNodeBuilder {
    pub fn new() -> Self {
        Self {
            keypair: None,
            listening_address: None,
            bootstrap_nodes: None,
            indentify_certificate: None,
            gossipsub_topics: None,
        }
    }
    pub fn with_keypair(self, keypair: Keypair) -> Self {
        Self {
            keypair: Some(keypair),
            ..self
        }
    }
    pub fn with_listening_address(self, listening_address: Multiaddr) -> Self {
        Self {
            listening_address: Some(listening_address),
            ..self
        }
    }
    pub fn with_bootstrap_nodes(self, bootstrap_nodes: HashSet<Multiaddr>) -> Self {
        Self {
            bootstrap_nodes: Some(bootstrap_nodes),
            ..self
        }
    }
    pub fn with_indentify_certificate(self, indentify_certificate: AuthorityCertificate) -> Self {
        Self {
            indentify_certificate: Some(indentify_certificate),
            ..self
        }
    }
    pub fn with_gossipsub_topics(self, gossipsub_topics: HashSet<String>) -> Self {
        Self {
            gossipsub_topics: Some(gossipsub_topics),
            ..self
        }
    }
    pub fn build(
        self,
    ) -> anyhow::Result<(
        P2pNode,
        tokio::sync::broadcast::Receiver<Message>,
        tokio::sync::mpsc::Sender<P2pRequest>,
    )> {
        let keypair = match self.keypair {
            Some(keypair) => keypair,
            None => {
                tracing::warn!("No keypair provided for node, generatng a new keypair");
                Keypair::generate_ed25519()
            }
        };
        let listening_address = match self.listening_address {
            Some(listening_address) => listening_address,
            None => {
                tracing::warn!("No listening address provided for node, using default");
                "/ip4/0.0.0.0"
                    .parse::<Multiaddr>()
                    .unwrap() //safe because constant
                    .with(Protocol::Tcp(DEFAULT_LISTENING_PORT))
            }
        };
        let bootstrap_nodes = match self.bootstrap_nodes {
            Some(bootstrap_nodes) => bootstrap_nodes,
            None => {
                tracing::warn!("No bootstrap nodes provided for node, using empty set");
                HashSet::new()
            }
        };
        let gossipsub_topics = match self.gossipsub_topics {
            Some(gossipsub_topics) => gossipsub_topics,
            None => {
                tracing::warn!("No gossipsub topics provided for node, using empty set");
                HashSet::new()
            }
        };
        P2pNode::new(
            keypair,
            listening_address,
            bootstrap_nodes,
            self.indentify_certificate,
            gossipsub_topics,
        )
    }
}
