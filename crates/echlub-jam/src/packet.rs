//! Binary wire format for the jam prototype.
//!
//! Layout (little endian): `"EJ"` magic, version `u8`, kind `u8`, peer `u8`,
//! then for audio: seq `u32`, sample_time `u64`, sample count `u16`, PCM `i16`s.

pub const MAGIC: [u8; 2] = *b"EJ";
pub const VERSION: u8 = 1;
/// Peer id the relay uses for mixes it sends back to clients.
pub const RELAY_PEER_ID: u8 = 255;

const KIND_HELLO: u8 = 1;
const KIND_BYE: u8 = 2;
const KIND_AUDIO: u8 = 3;
const HEADER_LEN: usize = 5;
const AUDIO_HEADER_LEN: usize = HEADER_LEN + 4 + 8 + 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioFrame {
    pub peer_id: u8,
    pub seq: u32,
    /// Sender clock position of the first sample, in samples.
    pub sample_time: u64,
    pub samples: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Packet {
    Hello { peer_id: u8 },
    Bye { peer_id: u8 },
    Audio(AudioFrame),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PacketError {
    #[error("packet too short")]
    TooShort,
    #[error("bad magic")]
    BadMagic,
    #[error("unsupported version {0}")]
    Version(u8),
    #[error("unknown kind {0}")]
    Kind(u8),
    #[error("sample count does not match payload length")]
    Length,
}

impl Packet {
    pub fn encode(&self) -> Vec<u8> {
        let (kind, peer) = match self {
            Packet::Hello { peer_id } => (KIND_HELLO, *peer_id),
            Packet::Bye { peer_id } => (KIND_BYE, *peer_id),
            Packet::Audio(f) => (KIND_AUDIO, f.peer_id),
        };
        let mut out = Vec::with_capacity(match self {
            Packet::Audio(f) => AUDIO_HEADER_LEN + f.samples.len() * 2,
            _ => HEADER_LEN,
        });
        out.extend_from_slice(&MAGIC);
        out.push(VERSION);
        out.push(kind);
        out.push(peer);
        if let Packet::Audio(f) = self {
            out.extend_from_slice(&f.seq.to_le_bytes());
            out.extend_from_slice(&f.sample_time.to_le_bytes());
            out.extend_from_slice(&(f.samples.len() as u16).to_le_bytes());
            for s in &f.samples {
                out.extend_from_slice(&s.to_le_bytes());
            }
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Packet, PacketError> {
        if bytes.len() < HEADER_LEN {
            return Err(PacketError::TooShort);
        }
        if bytes[0..2] != MAGIC {
            return Err(PacketError::BadMagic);
        }
        if bytes[2] != VERSION {
            return Err(PacketError::Version(bytes[2]));
        }
        let peer_id = bytes[4];
        match bytes[3] {
            KIND_HELLO => Ok(Packet::Hello { peer_id }),
            KIND_BYE => Ok(Packet::Bye { peer_id }),
            KIND_AUDIO => {
                if bytes.len() < AUDIO_HEADER_LEN {
                    return Err(PacketError::TooShort);
                }
                let seq = u32::from_le_bytes(bytes[5..9].try_into().expect("len"));
                let sample_time = u64::from_le_bytes(bytes[9..17].try_into().expect("len"));
                let count = u16::from_le_bytes(bytes[17..19].try_into().expect("len")) as usize;
                let payload = &bytes[AUDIO_HEADER_LEN..];
                if payload.len() != count * 2 {
                    return Err(PacketError::Length);
                }
                let samples = payload
                    .chunks_exact(2)
                    .map(|c| i16::from_le_bytes([c[0], c[1]]))
                    .collect();
                Ok(Packet::Audio(AudioFrame {
                    peer_id,
                    seq,
                    sample_time,
                    samples,
                }))
            }
            other => Err(PacketError::Kind(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_round_trip() {
        let p = Packet::Audio(AudioFrame {
            peer_id: 3,
            seq: 42,
            sample_time: 5376,
            samples: vec![0, 1, -1, i16::MAX, i16::MIN],
        });
        assert_eq!(Packet::decode(&p.encode()).unwrap(), p);
    }

    #[test]
    fn control_round_trip() {
        for p in [Packet::Hello { peer_id: 1 }, Packet::Bye { peer_id: 2 }] {
            assert_eq!(Packet::decode(&p.encode()).unwrap(), p);
        }
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(Packet::decode(b"xx"), Err(PacketError::TooShort));
        assert_eq!(
            Packet::decode(b"XX\x01\x01\x00"),
            Err(PacketError::BadMagic)
        );
        assert_eq!(
            Packet::decode(b"EJ\x09\x01\x00"),
            Err(PacketError::Version(9))
        );
        let mut bytes = Packet::Audio(AudioFrame {
            peer_id: 0,
            seq: 0,
            sample_time: 0,
            samples: vec![1, 2],
        })
        .encode();
        bytes.pop();
        assert_eq!(Packet::decode(&bytes), Err(PacketError::Length));
    }
}
