use std::net::SocketAddr;
use crate::{
    dev_tool::{Transaction, PeerId},
    transport::PeerConnection,
    message::NetMessage,
};

use super::types::{ForwardInfo, InboundGwJoinRequest, AcceptedTracker};

#[derive(Debug)]
pub(crate) enum Event {
    // todo: instead of returning InboundJoinReq which is an internal event
    // return a proper well formed ConnectOp and any other types needed (PeerConnection etc.)
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
        error: HandshakeError,
    },
    /// An outbound connection to a gateway was rejected.
    OutboundGatewayConnectionRejected { peer_id: PeerId },
    /// An inbound connection in a gateway was rejected.
    InboundConnectionRejected { peer_id: PeerId },
    /// An outbound connection to a gateway was successfully established. It can be managed by the connection manager.
    OutboundGatewayConnectionSuccessful {
        peer_id: PeerId,
        connection: PeerConnection,
        remaining_checks: usize,
    },
    /// Clean up a transaction that was completed or duplicate.
    RemoveTransaction(Transaction),
    /// Wait for replies via an other peer from forwarded connection attempts.
    TransientForwardTransaction {
        target: SocketAddr,
        tx: Transaction,
        forward_to: PeerId,
        msg: Box<NetMessage>,
    },
}

#[derive(Debug)]
pub(crate) enum InternalEvent {
    InboundGwJoinRequest(InboundGwJoinRequest),
    /// Regular connection established
    OutboundConnEstablished(PeerId, PeerConnection),
    OutboundGwConnEstablished(PeerId, PeerConnection),
    OutboundGwConnConfirmed(AcceptedTracker),
    DropInboundConnection(SocketAddr),
    RemoteConnectionAttempt {
        remote: PeerId,
        tracker: AcceptedTracker,
    },
    NextCheck(AcceptedTracker),
    FinishedOutboundConnProcess(AcceptedTracker),
}

pub(crate) enum ExternConnection {
    Establish {
        peer: PeerId,
        tx: Transaction,
        is_gw: bool,
    },
    Dropped {
        peer: PeerId,
    },
}
