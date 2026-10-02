//! UDP jam relay prototype.
//!
//! Two modes: `mix` jitter-buffers each peer and sends one mix-minus per peer
//! on a fixed frame clock; `forward` relays each audio packet to every other
//! peer on arrival and leaves buffering and mixing to clients.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use echlub_jam::jitter::{JitterBuffer, JitterStats};
use echlub_jam::mixer::mix_minus;
use echlub_jam::packet::{AudioFrame, Packet, RELAY_PEER_ID};
use echlub_jam::{frame_ms, FRAME_SAMPLES};

/// Longest sleep between socket polls.
pub const POLL_INTERVAL: Duration = Duration::from_micros(250);

#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// Forward packets on arrival instead of mixing on the relay clock.
    pub forward: bool,
    pub jitter_depth: usize,
    pub max_peers: usize,
    pub peer_timeout: Duration,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            forward: false,
            jitter_depth: 2,
            max_peers: 4,
            peer_timeout: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RelayStats {
    pub ticks: u64,
    pub late_ticks: u64,
    pub packets_in: u64,
    pub packets_out: u64,
    pub decode_errors: u64,
    pub rejected: u64,
    /// Final jitter stats for peers still connected at shutdown.
    pub peers: Vec<(u8, JitterStats)>,
}

struct PeerSlot {
    id: u8,
    addr: SocketAddr,
    jb: JitterBuffer,
    last_seen: Instant,
}

pub struct Relay {
    config: RelayConfig,
    peers: Vec<PeerSlot>,
    stats: RelayStats,
}

impl Relay {
    pub fn new(config: RelayConfig) -> Self {
        Self {
            config,
            peers: Vec::new(),
            stats: RelayStats::default(),
        }
    }

    fn slot(&mut self, id: u8, addr: SocketAddr, now: Instant) -> Option<&mut PeerSlot> {
        if let Some(i) = self.peers.iter().position(|p| p.addr == addr) {
            self.peers[i].last_seen = now;
            self.peers[i].id = id;
            return Some(&mut self.peers[i]);
        }
        if self.peers.len() >= self.config.max_peers {
            self.stats.rejected += 1;
            return None;
        }
        self.peers.push(PeerSlot {
            id,
            addr,
            jb: JitterBuffer::new(self.config.jitter_depth, FRAME_SAMPLES),
            last_seen: now,
        });
        self.peers.last_mut()
    }

    /// Handle one datagram; returns datagrams to send immediately (forward mode).
    pub fn handle(
        &mut self,
        bytes: &[u8],
        from: SocketAddr,
        now: Instant,
    ) -> Vec<(SocketAddr, Vec<u8>)> {
        self.stats.packets_in += 1;
        match Packet::decode(bytes) {
            Ok(Packet::Hello { peer_id }) => {
                self.slot(peer_id, from, now);
            }
            Ok(Packet::Bye { .. }) => self.peers.retain(|p| p.addr != from),
            Ok(Packet::Audio(frame)) => {
                let forward = self.config.forward;
                if let Some(slot) = self.slot(frame.peer_id, from, now) {
                    if forward {
                        return self
                            .peers
                            .iter()
                            .filter(|p| p.addr != from)
                            .map(|p| (p.addr, bytes.to_vec()))
                            .collect();
                    }
                    slot.jb.push(frame.seq, frame.samples);
                }
            }
            Err(_) => self.stats.decode_errors += 1,
        }
        Vec::new()
    }

    /// One frame clock tick: returns datagrams to send.
    pub fn tick(&mut self, seq: u32, now: Instant) -> Vec<(SocketAddr, Vec<u8>)> {
        let timeout = self.config.peer_timeout;
        self.peers
            .retain(|p| now.duration_since(p.last_seen) < timeout);
        self.stats.ticks += 1;
        if self.config.forward {
            return Vec::new();
        }
        let inputs: Vec<Vec<i16>> = self.peers.iter_mut().map(|p| p.jb.pop().1).collect();
        let mixes = mix_minus(&inputs, FRAME_SAMPLES);
        self.peers
            .iter()
            .zip(mixes)
            .map(|(p, samples)| {
                let packet = Packet::Audio(AudioFrame {
                    peer_id: RELAY_PEER_ID,
                    seq,
                    sample_time: seq as u64 * FRAME_SAMPLES as u64,
                    samples,
                });
                (p.addr, packet.encode())
            })
            .collect()
    }

    pub fn stats(&self) -> RelayStats {
        let mut s = self.stats.clone();
        s.peers = self.peers.iter().map(|p| (p.id, p.jb.stats())).collect();
        s
    }
}

/// Run the relay on `socket` until `stop` is set.
pub fn run(socket: UdpSocket, config: RelayConfig, stop: &AtomicBool) -> io::Result<RelayStats> {
    let mut relay = Relay::new(config);
    let period = Duration::from_secs_f64(frame_ms() / 1000.0);
    let start = Instant::now();
    let mut seq: u32 = 0;
    let mut buf = [0u8; 2048];
    socket.set_nonblocking(true)?;
    while !stop.load(Ordering::Relaxed) {
        let deadline = start + period * seq;
        let now = Instant::now();
        if now >= deadline {
            if now - deadline > period {
                relay.stats.late_ticks += 1;
            }
            for (addr, bytes) in relay.tick(seq, now) {
                if socket.send_to(&bytes, addr).is_ok() {
                    relay.stats.packets_out += 1;
                }
            }
            seq = seq.wrapping_add(1);
            // Far behind (suspended process): skip ahead instead of bursting.
            let behind = (Instant::now() - start).as_secs_f64() / period.as_secs_f64();
            if behind - seq as f64 > 50.0 {
                seq = behind as u32;
            }
            continue;
        }
        // Drain everything queued, then sleep briefly. Socket read timeouts are
        // too coarse on some platforms (8 ms in gVisor) to clock audio with.
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, from)) => {
                    for (addr, bytes) in relay.handle(&buf[..len], from, Instant::now()) {
                        if socket.send_to(&bytes, addr).is_ok() {
                            relay.stats.packets_out += 1;
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                // ICMP port unreachable from a departed peer surfaces here on some OSes.
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {}
                Err(e) => return Err(e),
            }
        }
        thread::sleep((deadline - now).min(POLL_INTERVAL));
    }
    Ok(relay.stats())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], port))
    }

    fn audio(peer: u8, seq: u32, v: i16) -> Vec<u8> {
        Packet::Audio(AudioFrame {
            peer_id: peer,
            seq,
            sample_time: 0,
            samples: vec![v; FRAME_SAMPLES],
        })
        .encode()
    }

    #[test]
    fn mixes_minus_one_per_peer() {
        let mut relay = Relay::new(RelayConfig {
            jitter_depth: 1,
            ..Default::default()
        });
        let now = Instant::now();
        relay.handle(&audio(0, 0, 10), addr(1), now);
        relay.handle(&audio(1, 0, 100), addr(2), now);
        let out = relay.tick(0, now);
        assert_eq!(out.len(), 2);
        let decode = |b: &[u8]| match Packet::decode(b).unwrap() {
            Packet::Audio(f) => f.samples[0],
            _ => panic!(),
        };
        assert_eq!(decode(&out[0].1), 100);
        assert_eq!(decode(&out[1].1), 10);
    }

    #[test]
    fn forward_mode_relays_to_others_only() {
        let mut relay = Relay::new(RelayConfig {
            forward: true,
            ..Default::default()
        });
        let now = Instant::now();
        relay.handle(&Packet::Hello { peer_id: 0 }.encode(), addr(1), now);
        relay.handle(&Packet::Hello { peer_id: 1 }.encode(), addr(2), now);
        relay.handle(&Packet::Hello { peer_id: 2 }.encode(), addr(3), now);
        let packet = audio(0, 7, 5);
        let out = relay.handle(&packet, addr(1), now);
        let targets: Vec<_> = out.iter().map(|(a, _)| *a).collect();
        assert_eq!(targets, vec![addr(2), addr(3)]);
        assert!(out.iter().all(|(_, b)| *b == packet));
        assert!(relay.tick(0, now).is_empty());
    }

    #[test]
    fn rejects_beyond_capacity_and_counts_garbage() {
        let mut relay = Relay::new(RelayConfig {
            max_peers: 1,
            ..Default::default()
        });
        let now = Instant::now();
        relay.handle(&Packet::Hello { peer_id: 0 }.encode(), addr(1), now);
        relay.handle(&Packet::Hello { peer_id: 1 }.encode(), addr(2), now);
        relay.handle(b"junk", addr(3), now);
        let s = relay.stats();
        assert_eq!(s.rejected, 1);
        assert_eq!(s.decode_errors, 1);
        assert_eq!(s.peers.len(), 1);
    }

    #[test]
    fn bye_and_timeout_remove_peers() {
        let mut relay = Relay::new(RelayConfig {
            peer_timeout: Duration::from_millis(10),
            ..Default::default()
        });
        let now = Instant::now();
        relay.handle(&Packet::Hello { peer_id: 0 }.encode(), addr(1), now);
        relay.handle(&Packet::Hello { peer_id: 1 }.encode(), addr(2), now);
        relay.handle(&Packet::Bye { peer_id: 0 }.encode(), addr(1), now);
        assert_eq!(relay.stats().peers.len(), 1);
        relay.tick(0, now + Duration::from_millis(20));
        assert!(relay.stats().peers.is_empty());
    }
}
