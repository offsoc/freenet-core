use std::net::SocketAddr;
use tokio::sync::mpsc;
use crate::{
    dev_tool::{PeerId, Transaction},
    message::NetMessage,
    transport::TransportError,
};

pub(super) type Result<T, E = HandshakeError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub(super) enum HandshakeError {
    #[error("channel closed")]
    ChannelClosed,
    #[error("connection closed to {0}")]
    ConnectionClosed(SocketAddr),
    #[error(transparent)]
    Serialization(#[from] Box<bincode::ErrorKind>),
    #[error(transparent)]
    TransportError(#[from] TransportError),
    #[error("receibed an unexpected message at this point: {0}")]
    UnexpectedMessage(Box<NetMessage>),
}

/// Use for sending messages to a peer which has not yet been confirmed at a logical level
/// or is just a transient connection (e.g. in case of gateways just forwarding messages).
pub(super) struct OutboundMessage(pub(super) mpsc::Sender<(SocketAddr, NetMessage)>);

impl OutboundMessage {
    pub async fn send_to(&self, remote: SocketAddr, msg: NetMessage) -> Result<()> {
        self.0
            .send((remote, msg))
            .await
            .map_err(|_| HandshakeError::ChannelClosed)?;
        Ok(())
    }
}

pub(super) enum ExternConnection {
    Establish {
        peer: PeerId,
        tx: Transaction,
        is_gw: bool,
    },
    Dropped {
        peer: PeerId,
    },
}

/// Used for communicating with the HandshakeHandler.
pub(super) struct HanshakeHandlerMsg(pub(crate) mpsc::Sender<ExternConnection>);

impl HanshakeHandlerMsg {
    pub async fn establish_conn(&self, remote: PeerId, tx: Transaction, is_gw: bool) -> Result<()> {
        self.0
            .send(ExternConnection::Establish {
                peer: remote,
                tx,
                is_gw,
            })
            .await
            .map_err(|_| HandshakeError::ChannelClosed)?;
        Ok(())
    }

    pub async fn drop_connection(&self, remote: PeerId) -> Result<()> {
        self.0
            .send(ExternConnection::Dropped { peer: remote })
            .await
            .map_err(|_| HandshakeError::ChannelClosed)?;
        Ok(())
    }
}
