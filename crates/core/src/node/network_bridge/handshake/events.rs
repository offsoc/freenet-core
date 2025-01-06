use std::net::SocketAddr;
use crate::{
    dev_tool::{PeerId, Transaction},
    message::NetMessage,
    transport::PeerConnection,
    operations::connect::ConnectOp,
};

#[derive(Debug)]
pub(super) struct ForwardInfo {
    pub target: PeerId,
    pub msg: NetMessage,
}

#[derive(Debug)]
pub(super) enum Event {
    /// An inbound connection to a peer was successfully established at a gateway.
    InboundConnection {
        id: Transaction,
        conn: PeerConnection,
        joiner: PeerId,
        op: Option<Box<ConnectOp>>,
        forward_info: Option<Box<ForwardInfo>>,
    },
    /// An outbound connection to a peer was successfully established.
    OutboundConnectionSuccessful {
        peer_id: PeerId,
        connection: PeerConnection,
    },
    /// An outbound connection to a peer failed to be established.
    OutboundConnectionFailed {
        peer_id: PeerId,
        error: super::types::HandshakeError,
    },
    /// An outbound connection to a gateway was rejected.
    OutboundGatewayConnectionRejected { peer_id: PeerId },
    /// An inbound connection in a gateway was rejected.
    InboundConnectionRejected { peer_id: PeerId },
    /// An outbound connection to a gateway was successfully established.
    OutboundGatewayConnectionSuccessful {
        peer_id: PeerId,
        connection: PeerConnection,
        remaining_checks: usize,
    },
    /// Clean up a transaction that was completed or duplicate.
    RemoveTransaction(Transaction),
    /// Wait for replies via another peer from forwarded connection attempts.
    TransientForwardTransaction {
        target: SocketAddr,
        tx: Transaction,
        forward_to: PeerId,
        msg: Box<NetMessage>,
    },
}

#[derive(Debug)]
pub(super) enum InternalEvent {
    InboundGwJoinRequest(super::connection::InboundGwJoinRequest),
    /// Regular connection established
    OutboundConnEstablished(PeerId, PeerConnection),
    OutboundGwConnEstablished(PeerId, PeerConnection),
    OutboundGwConnConfirmed(super::connection::AcceptedTracker),
    DropInboundConnection(SocketAddr),
    RemoteConnectionAttempt {
        remote: PeerId,
        tracker: super::connection::AcceptedTracker,
    },
    NextCheck(super::connection::AcceptedTracker),
    FinishedOutboundConnProcess(super::connection::AcceptedTracker),
}
