use crate::message::NetMessage;
use crate::transport::TransportError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Unexpected message received: {0:?}")]
    UnexpectedMessage(Box<NetMessage>),
    
    #[error("Connection closed to {0}")]
    ConnectionClosed(std::net::SocketAddr),
    
    #[error("Serialization error: {0:?}")]
    Serialization(Option<bincode::Error>),
}

pub fn decode_msg(data: &[u8]) -> Result<NetMessage, HandshakeError> {
    bincode::deserialize(data).map_err(|err| HandshakeError::Serialization(Some(err)))
}
