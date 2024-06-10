//! This library implements the Ergo platform handshake message
//! protocol.
//!
//! The handshake process is describes  [here](https://docs.ergoplatform.com/dev/p2p/p2p-handshake).
//!
//! This library itself doesn't use any timeout to operate, but as it is
//! advise in communicating with any third party service, All call should be wrapped
//! in a timeout. By not using an internal timeout feature, we keep the API and dependencies
//! minimal and most importantly let users choose what library and strategy they are most
//! comfortable with.
//!
//! ```ignore
//! use p2p_handshake::{handshake, Version};
//!
//! handshake("127.0.0.1:90:30",  "agent-name", Version([2,1,3]), |_stream, msg| {
//!     println!("Reply: {:?}", msg);  
//!     Ok(())
//! }).await;
//! ```
//!
mod encoder;
mod error;

pub use encoder::{HandshakeMessage, TinyString, Version};
use error::{ProtocolError, ProtocolResult};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

/// Handshake implements the p2p handshake portion of Ergo platform protocol
///
///
/// * `target_address` - The address and port of this target node (ex. 127.0.0.1:9030).
/// * `agent_name` - The name of this client making the request
/// * `version` - The version of this client making the request
/// * `on_accept` - A callback that gets called when the handshake is successful.
///
pub async fn handshake<A: ToSocketAddrs, F>(
    target_address: A,
    agent_name: &str,
    version: Version,
    on_accept: F,
) -> ProtocolResult<()>
where
    F: FnOnce(TcpStream, HandshakeMessage) -> ProtocolResult<()>,
{
    // Making the connection
    let mut stream = TcpStream::connect(target_address).await?;

    // Compose the request and send to the wire.
    let request = HandshakeMessage {
        agent_name: agent_name.try_into().map_err(ProtocolError::Unknown)?,
        version,
        peer_name: TinyString("evan-testnet".into()),
    };

    let data = request.encode_for_request()?;
    stream.write_all(&data).await?;

    // Read just enough data from the wire to extract the target response.
    let mut raw_response = vec![0; 255];
    stream.read(&mut raw_response).await?;
    let response = HandshakeMessage::decode_from_response(raw_response)?;

    on_accept(stream, response)
}
