use std::{collections::{HashMap, HashSet}, net::SocketAddr, sync::Arc};
use parking_lot::RwLock;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    dev_tool::{Transaction, PeerId},
    message::NetMessage,
    operations::connect::ConnectivityInfo,
    ring::{ConnectionManager, PeerKeyLocation},
    router::Router,
    transport::{InboundConnectionHandler, OutboundConnectionHandler, PeerConnection},
};

pub(crate) struct ClientResponsesReceiver(pub(crate) UnboundedReceiver<(PeerId, crate::client_events::HostResult)>);

pub(crate) fn client_responses_channel() -> (ClientResponsesReceiver, ClientResponsesSender) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ClientResponsesReceiver(rx), ClientResponsesSender(tx))
}

impl std::ops::Deref for ClientResponsesReceiver {
    type Target = UnboundedReceiver<(PeerId, crate::client_events::HostResult)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ClientResponsesReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub(crate) struct ClientResponsesSender(pub(crate) UnboundedSender<(PeerId, crate::client_events::HostResult)>);

impl std::ops::Deref for ClientResponsesSender {
    type Target = UnboundedSender<(PeerId, crate::client_events::HostResult)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct ForwardInfo {
    pub target: PeerId,
    pub msg: NetMessage,
}

#[derive(Debug)]
pub(crate) struct InboundGwJoinRequest {
    pub conn: PeerConnection,
    pub id: Transaction,
    pub joiner: PeerId,
    pub hops_to_live: usize,
    pub max_hops_to_live: usize,
    pub skip_list: Vec<PeerId>,
}

pub(crate) struct TransientConnection {
    pub tx: Transaction,
    pub joiner: PeerId,
    pub max_hops_to_live: usize,
    pub hops_to_live: usize,
    pub skip_list: Vec<PeerId>,
}

impl TransientConnection {
    pub fn is_drop_connection_message(&self, net_message: &NetMessage) -> bool {
        if let NetMessage::V1(NetMessageV1::Connect(ConnectMsg::Request {
            id,
            msg: ConnectRequest::CleanConnection { joiner },
            ..
        })) = net_message
        {
            // this peer should never be receiving messages for other transactions or other peers at this point
            debug_assert_eq!(id, &self.tx);
            debug_assert_eq!(joiner.peer, self.joiner);

            if id != &self.tx || joiner.peer != self.joiner {
                return false;
            }
            return true;
        }
        false
    }
}

pub(crate) struct HandshakeHandler {
    /// Tracks ongoing connection attempts by their remote socket address
    pub connecting: HashMap<SocketAddr, Transaction>,

    /// Set of socket addresses for established connections  
    pub connected: HashSet<SocketAddr>,

    /// Handles incoming connections from the network
    pub inbound_conn_handler: InboundConnectionHandler,

    /// Initiates outgoing connections to remote peers
    pub outbound_conn_handler: OutboundConnectionHandler,

    /// Queue of ongoing outbound connection attempts
    /// Used for non-gateway peers initiating connections
    pub ongoing_outbound_connections: FuturesUnordered<BoxFuture<'static, OutboundConnResult>>,

    /// Queue of inbound connections not yet confirmed at the logical level
    /// Used primarily by gateways for handling new peer join requests
    pub unconfirmed_inbound_connections: FuturesUnordered<
        BoxFuture<'static, Result<(InternalEvent, PeerOutboundMessage), HandshakeError>>,
    >,

    /// Mapping of socket addresses to channels for sending messages to peers
    /// Used for both confirmed and unconfirmed connections
    pub outbound_messages: HashMap<SocketAddr, OutboundMessageSender>,

    /// Receiver for messages to be sent to peers not yet confirmed
    /// Part of the OutboundMessage public API
    pub pending_msg_rx: OutboundMessageReceiver,

    /// Receiver for commands to establish new outbound connections
    /// Part of the EstablishConnection public API
    pub establish_connection_rx: EstablishConnectionReceiver,

    /// Manages the node's connections and topology
    pub connection_manager: ConnectionManager,

    /// Handles routing decisions within the network
    pub router: Arc<RwLock<Router>>,
}

pub(crate) type OutboundMessageSender = mpsc::Sender<NetMessage>;
pub(crate) type OutboundMessageReceiver = mpsc::Receiver<(SocketAddr, NetMessage)>;
pub(crate) type EstablishConnectionReceiver = mpsc::Receiver<ExternConnection>;

#[repr(transparent)]
pub(crate) struct PeerOutboundMessage(pub mpsc::Receiver<NetMessage>);

#[derive(Debug)]
pub(crate) struct AcceptedTracker {
    pub gw_peer: PeerKeyLocation,
    pub gw_conn: PeerConnection,
    pub gw_accepted_processed: bool,
    pub gw_accepted: bool,
    /// Remaining checks to be made, at max total_checks
    pub remaining_checks: usize,
    /// At max this will be total_checks
    pub accepted: usize,
    /// Equivalent to max_hops_to_live
    pub total_checks: usize,
    pub tx: Transaction,
}
