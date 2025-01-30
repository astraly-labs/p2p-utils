pub enum P2pRequest {
    Broadcast(String, Vec<u8>),
}

pub struct ReceivedConnection {
    pub connection_id: usize,
    pub peer_id: String,
    pub pubkey: [u8; 32],
    pub listen_addrs: Vec<String>,
    pub observed_addr: String,
    pub certificate: Option<String>,
}