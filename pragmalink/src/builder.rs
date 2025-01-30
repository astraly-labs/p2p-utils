use crate::{
    types::{P2pRequest, ReceivedConnection, ReceivedMessage}, P2pNode, DEFAULT_LISTENING_PORT
};
use libp2p::{Multiaddr, identity::Keypair, multiaddr::Protocol};
use std::collections::HashSet;

pub struct P2pNodeBuilder {
    keypair: Option<String>,
    listening_address: Option<String>,
    bootstrap_nodes: Option<HashSet<String>>,
    indentify_certificate: Option<String>,
    gossipsub_topics: Option<HashSet<String>>,
}

impl P2pNodeBuilder {
    /// Create a new P2pNodeBuilder: all set to None
    pub fn new() -> Self {
        Self {
            keypair: None,
            listening_address: None,
            bootstrap_nodes: None,
            indentify_certificate: None,
            gossipsub_topics: None,
        }
    }
    /// Define an ed25519 keypair, encoded in hexadecimals
    pub fn with_keypair(self, keypair: String) -> Self {
        Self {
            keypair: Some(keypair),
            ..self
        }
    }
    /// Define a listening address: example: "/ip4/0.0.0.0/tcp/1123"
    pub fn with_listening_address(self, listening_address: String) -> Self {
        Self {
            listening_address: Some(listening_address),
            ..self
        }
    }
    /// Define a set of bootstrap nodes addresses
    pub fn with_bootstrap_nodes(self, bootstrap_nodes: HashSet<String>) -> Self {
        Self {
            bootstrap_nodes: Some(bootstrap_nodes),
            ..self
        }
    }
    /// Define an identity information that will be used to identify the node to other peers
    pub fn with_indentify_certificate(self, indentify_certificate: String) -> Self {
        Self {
            indentify_certificate: Some(indentify_certificate),
            ..self
        }
    }
    /// Define a set of gossipsub topics, with the names of the topics
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
        tokio::sync::broadcast::Receiver<ReceivedMessage>,
        tokio::sync::mpsc::Sender<P2pRequest>,
        tokio::sync::mpsc::Receiver<(ReceivedConnection, tokio::sync::oneshot::Sender<bool>)>,
    )> {
        let keypair = match self.keypair {
            Some(keypair) => Keypair::ed25519_from_bytes(hex::decode(keypair)?)
                .map_err(|e| anyhow::anyhow!("Invalid keypair provided: {}", e))?,
            None => {
                tracing::warn!("No keypair provided for node, generatng a new keypair");
                Keypair::generate_ed25519()
            }
        };
        let listening_address = match self.listening_address {
            Some(listening_address) => listening_address.parse::<Multiaddr>()?,
            None => {
                tracing::warn!("No listening address provided for node, using default");
                "/ip4/0.0.0.0".parse::<Multiaddr>()
                .unwrap() //safe because constant
                .with(Protocol::Tcp(DEFAULT_LISTENING_PORT))
            }
        };
        let bootstrap_nodes = match self.bootstrap_nodes {
            Some(bootstrap_nodes) => bootstrap_nodes.into_iter().map(|addr| addr.parse::<Multiaddr>()).collect::<Result<HashSet<Multiaddr>, _>>()?,
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
