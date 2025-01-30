use crate::behavior::P2pBehavior;
use crate::types::P2pRequest;
use libp2p::{Multiaddr, PeerId, Swarm, futures::StreamExt, identity::Keypair};
use libp2p_gossipsub::{IdentTopic, TopicHash};
use std::collections::{HashMap, HashSet};
use types::{ReceivedConnection, ReceivedMessage};

pub mod builder;
pub mod traits;
pub mod types;
mod behavior;
mod events;

const DEFAULT_LISTENING_PORT: u16 = 1123;
const CHANNEL_SIZE: usize = 1000;

/// A P2P network node, use kademlia DHT and gossipsub protocol
pub struct P2pNode {
    /// The node ed25519 keypair
    pub keypair: Keypair,
    /// The node peer id
    pub peer_id: PeerId,
    pub swarm: Swarm<P2pBehavior>,
    /// Connected peers
    pub peers: HashSet<PeerId>,
    /// Subscribed gossipsub topics with name and hash
    pub gossipsub_topics: HashMap<TopicHash, String>,
    /// The address the node is listening on
    pub listening_address: Multiaddr,
    /// An optional certificate, can be used to identify the node
    pub identify_certificate: Option<String>,
    /// The bootstrap nodes addresses, to enter the network and discover other peers
    pub bootstrap_nodes: HashSet<Multiaddr>,
    /// The channel to send received messages, with payload, source, topic...
    pub received_messages_tx: tokio::sync::broadcast::Sender<ReceivedMessage>,
    /// The channel to receive messages to send to other peers
    pub send_messages_rx: tokio::sync::mpsc::Receiver<P2pRequest>,
    /// Used to send the informations about a peers that sent a connection request and a oneshot channel to wait for the authorization (true / false)
    pub connection_authorization_tx:
        tokio::sync::mpsc::Sender<(ReceivedConnection, tokio::sync::oneshot::Sender<bool>)>,
}

impl P2pNode {
    pub fn new(
        keypair: Keypair,
        listening_address: Multiaddr,
        bootstrap_nodes: HashSet<Multiaddr>,
        identify_certificate: Option<String>,
        gossipsub_topics: HashSet<String>,
    ) -> anyhow::Result<(
        Self,
        tokio::sync::broadcast::Receiver<ReceivedMessage>,
        tokio::sync::mpsc::Sender<P2pRequest>,
        tokio::sync::mpsc::Receiver<(ReceivedConnection, tokio::sync::oneshot::Sender<bool>)>,
    )> {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair.clone())
            .with_tokio()
            .with_tcp(
                Default::default(),
                (libp2p::tls::Config::new, libp2p::noise::Config::new),
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|identity| {
                P2pBehavior::new(identity.clone(), identify_certificate.clone()).unwrap() //safe: TODO: remove unwrap
            })?
            .build();

        let (received_messages_tx, received_messages_rx) =
            tokio::sync::broadcast::channel(CHANNEL_SIZE);
        let (send_messages_tx, send_messages_rx) = tokio::sync::mpsc::channel(CHANNEL_SIZE);
        let (connection_authorization_tx, connection_authorization_rx) =
            tokio::sync::mpsc::channel(CHANNEL_SIZE);

        let mut sub_topics = HashMap::new();
        for topic in gossipsub_topics {
            let topic_id = libp2p_gossipsub::IdentTopic::new(&topic);
            swarm.behaviour_mut().gossipsub.subscribe(&topic_id)?;
            sub_topics.insert(topic_id.hash(), topic);
        }

        Ok((
            Self {
                send_messages_rx,
                keypair,
                peer_id: *swarm.local_peer_id(),
                swarm,
                listening_address,
                identify_certificate,
                bootstrap_nodes,
                received_messages_tx,
                peers: HashSet::new(),
                gossipsub_topics: sub_topics,
                connection_authorization_tx,
            },
            received_messages_rx,
            send_messages_tx,
            connection_authorization_rx,
        ))
    }
    /// Subscribe to a gossipsub topic of a given name, returning the hash of the topic
    pub fn subscribe_topic(&mut self, topic: &str) -> anyhow::Result<TopicHash> {
        let topic = libp2p_gossipsub::IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        Ok(topic.hash())
    }
    pub async fn run(mut self) -> anyhow::Result<()> {
        tracing::info!("Starting P2P node");
        self.swarm.listen_on(self.listening_address.clone())?;
        self.try_dial_bootstrap_nodes();
        loop {
            tokio::select! {
                    Some(req) = self.send_messages_rx.recv() => {
                        //TODO: handle error ??
                        let _ = self.handle_p2p_request(req);
                    }
                    event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
            }
        }
    }
    fn handle_p2p_request(&mut self, req: P2pRequest) -> anyhow::Result<()> {
        match req {
            P2pRequest::Broadcast(topic, data) => {
                let topic_id = IdentTopic::new(&topic);
                //TODO: warn log if topic not in subscriber topics
                match self.swarm.behaviour_mut().gossipsub.publish(topic_id, data) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to publish a message on topic {} : {}", topic, e)
                    }
                }
            }
        }
        Ok(())
    }
    fn try_dial_bootstrap_nodes(&mut self) {
        if self.bootstrap_nodes.is_empty() {
            tracing::warn!("No bootstrap nodes provided");
        } else {
            for addr in &self.bootstrap_nodes {
                match self.swarm.dial(addr.clone()) {
                    Err(e) => {
                        tracing::error!(
                            "Could not dial bootstrap node of address {}: {}",
                            addr.to_string(),
                            e
                        )
                    }
                    Ok(_) => {
                        tracing::info!("Dialed bootstrap node {}", addr.to_string())
                    }
                }
            }
        }
    }
}
