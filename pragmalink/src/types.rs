use libp2p_gossipsub::IdentTopic;

pub enum P2pRequest {
    Broadcast(IdentTopic, Vec<u8>),
}
