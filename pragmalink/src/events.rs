use crate::{P2pNode, behavior::P2pBehaviorEvent};
use anyhow::Context;
use libp2p::{identify, swarm::SwarmEvent};

impl P2pNode {
    pub async fn handle_swarm_event(
        &mut self,
        event: SwarmEvent<P2pBehaviorEvent>,
    ) -> anyhow::Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                let listen_address = address
                    .with_p2p(*self.swarm.local_peer_id())
                    .expect("Making multiaddr");
                tracing::info!("📡 Peer-to-peer listening on address {listen_address:?}");
            }
            SwarmEvent::Behaviour(behaviour) => match behaviour {
                P2pBehaviorEvent::Gossipsub(libp2p_gossipsub::Event::Message {
                    message, ..
                }) => {
                    self.received_messages_tx
                        .send(message)
                        .context("fatal error: messages rx send failed")?;
                }
                P2pBehaviorEvent::Identify(identify::Event::Received { peer_id, info, .. }) => {
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                    self.peers.insert(peer_id);
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, info.observed_addr);
                    tracing::info!("🤝 Peer {peer_id} accepted and added in kademlia peers");
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}
