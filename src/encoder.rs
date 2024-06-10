//! This module implements the Ergo platform handshake message
//! format, encoding and decoding feature.
//!
//! The description of the message format can be found [here](https://docs.ergoplatform.com/dev/p2p/p2p-handshake/#handshake-format).  
//!
//! Please note that this implementation is kept as minimal as possible
//! only including necessary parameters for performing a node handshake.
//!

use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::error::ProtocolError;
use crate::error::ProtocolResult;

use byteorder::ReadBytesExt;

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Version(pub [u8; 3]);

impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.0[0], self.0[1], self.0[2])
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts = value.split(".");
        let mut index = 0;
        let mut raw_version = [0u8; 3];
        for s in parts {
            if index >= 3 {
                return Err("Malformed version specs.".to_string());
            }
            let version_part = s
                .parse::<u8>()
                .map_err(|_| format!("Error parsing version component: `{}`.", s))?;
            raw_version[index] = version_part;
            index += 1;
        }

        if index != 3 {
            return Err("Malformed version specs.".to_string());
        }

        Ok(Version(raw_version))
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TinyString(pub String);

impl TryFrom<&str> for TinyString {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() > 255 {
            return Err("TinyString cannot hold more than 255 bytes.".to_string());
        }
        Ok(Self(value.to_string()))
    }
}

impl ToString for TinyString {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl Deref for TinyString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct HandshakeMessage {
    pub agent_name: TinyString,
    pub version: Version,
    pub peer_name: TinyString,
}

impl HandshakeMessage {
    pub fn encode_for_request(&self) -> ProtocolResult<Vec<u8>> {
        let mut buf = std::io::Cursor::new(vec![]);

        // The timestamp is encoded in Little Endian Base 128 also referred
        // VLQ (variable length quantity)
        leb128::write::unsigned(&mut buf, get_current_unix_timestamp())?;
        buf.write_all(&vec![self.agent_name.len() as u8])?;
        buf.write_all(self.agent_name.as_bytes())?;
        buf.write_all(&self.version.0)?;
        buf.write_all(&vec![self.peer_name.len() as u8])?;
        buf.write_all(self.peer_name.as_bytes())?;
        // We put `0`` to ignore peer_address parameter
        buf.write_all(&vec![0])?;
        Ok(buf.into_inner())
    }

    pub fn decode_from_response(data: Vec<u8>) -> ProtocolResult<Self> {
        let mut cursor = Cursor::new(data);
        let _timestamp = leb128::read::unsigned(&mut cursor).map_err(ProtocolError::LEB128Error)?;
        let agent_name = read_string(&mut cursor)?;
        let mut raw_version = [0u8; 3];
        cursor.read_exact(&mut raw_version)?;
        let peer_name = read_string(&mut cursor)?;

        Ok(HandshakeMessage {
            agent_name,
            version: Version(raw_version),
            peer_name,
        })
    }
}

fn get_current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected a valid unix epoch timestamp")
        .as_millis() as u64
}

fn read_string<R: Read>(reader: &mut R) -> ProtocolResult<TinyString> {
    let len: u8 = reader.read_u8()?;
    let mut buf = vec![0; len as usize];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf)
        .map_err(ProtocolError::Utf8Error)
        .map(|s| TinyString::try_from(s.as_str()))?
        .map_err(ProtocolError::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(Version::from_str("2.0.1").unwrap(), Version([2, 0, 1]));
        assert_eq!(
            Version::from_str("2.A.2").unwrap_err(),
            "Error parsing version component: `A`."
        );
        assert_eq!(
            Version::from_str("2.3.1.3").unwrap_err(),
            "Malformed version specs."
        );
        assert_eq!(
            Version::from_str("2.3").unwrap_err(),
            "Malformed version specs."
        )
    }

    #[test]
    fn test_tiny_string() {
        assert_eq!(TinyString::try_from("value").unwrap().to_string(), "value");

        let large_text = "x".repeat(300);
        assert_eq!(
            TinyString::try_from(large_text.as_str()).unwrap_err(),
            "TinyString cannot hold more than 255 bytes."
        );
    }

    #[test]
    fn test_encoding_decoding() -> ProtocolResult<()> {
        let handshake = HandshakeMessage {
            agent_name: TinyString("paul".to_string()),
            version: Version::from_str("3.2.1").expect("should extract version"),
            peer_name: TinyString("paul-node".to_string()),
        };

        let encoded_data = handshake.encode_for_request()?;
        let message = HandshakeMessage::decode_from_response(encoded_data)?;

        assert_eq!(message.agent_name, TinyString("paul".to_string()));
        assert_eq!(message.version.to_string(), "3.2.1".to_string());
        assert_eq!(message.peer_name, TinyString("paul-node".to_string()));

        Ok(())
    }
}
