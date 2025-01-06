use std::fmt::Display;
use crate::message::NetMessage;
use crate::transport::TransportError;

#[derive(Debug, thiserror::Error)]
pub(super) enum HandshakeError {
    #[error("channel closed")]
    ChannelClosed,
    #[error("connection closed to {0}")]
    ConnectionClosed(std::net::SocketAddr),
    #[error(transparent)]
    Serialization(#[from] Box<bincode::ErrorKind>),
    #[error(transparent)]
    TransportError(#[from] TransportError),
    #[error("received an unexpected message at this point: {0}")]
    UnexpectedMessage(Box<NetMessage>),
}

#[inline(always)]
pub(super) fn decode_msg(data: &[u8]) -> Result<NetMessage, HandshakeError> {
    bincode::deserialize(data).map_err(HandshakeError::Serialization)
}
