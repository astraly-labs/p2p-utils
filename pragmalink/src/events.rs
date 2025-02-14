use crate::{behavior::P2pBehaviorEvent, types::ReceivedMessage, P2pNode};
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
                    let topic_name = match self.gossipsub_topics.get(&message.topic) {
                        Some(topic) => topic,
                        None => {
                            tracing::warn!(
                                "Received a message on an unsubscribed topic",
                            );
                            return Ok(())
                        }
                    };

                    let received_message = ReceivedMessage {
                        source: message.source.map(|peer_id| peer_id.to_string()),
                        data: message.data.clone(),
                        topic: topic_name.clone(),
                    };

                    self.received_messages_tx
                        .send(received_message)
                        .context("fatal error: messages rx send failed")?;
                }
                P2pBehaviorEvent::Identify(identify::Event::Received {
                    peer_id,
                    info,
                    connection_id,
                }) => {
                    let connection_request = identify::Event::Received {
                        peer_id: peer_id.clone(),
                        info: info.clone(),
                        connection_id: connection_id.clone(),
                    }
                    .try_into();
                    let authorization_rx = match connection_request {
                        Ok(request) => {
                            let (tx, rx) = tokio::sync::oneshot::channel();
                            self.connection_authorization_tx
                                .send((request, tx))
                                .await
                                .context("fatal error: connection authorization tx send failed")?;
                            rx
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to convert identify event to connection request: {}",
                                e
                            );
                            return Ok(());
                        }
                    };

                    if authorization_rx
                        .await
                        .context("fatal error: connection authorization rx receive failed")?
                    {
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
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}
