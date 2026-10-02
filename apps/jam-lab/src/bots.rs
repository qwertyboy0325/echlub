//! Bot musicians that join a relay over real UDP sockets.
//!
//! Each bot runs its own frame clock: capture a frame, (optionally) delay or
//! drop it through an impairment model, send it; receive mixes, delay or drop
//! them likewise, jitter-buffer, and play out. In probe mode bots emit timing
//! clicks and measure one-way latency from the audio they hear; in music mode
//! they play their rock part and can record what they hear.
//!
//! All bots share one epoch, so latencies are comparable only within a single
//! process (e.g. localhost runs or impairment experiments).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use echlub_jam::jitter::{Concealment, JitterBuffer, JitterStats};
use echlub_jam::netsim::{ImpairedLink, LinkProfile};
use echlub_jam::packet::{AudioFrame, Packet};
use echlub_jam::probe::{OnsetDetector, ProbeSchedule};
use echlub_jam::{frame_ms, FRAME_SAMPLES, SAMPLE_RATE};
use echlub_jam_relay::{RelayConfig, RelayStats, POLL_INTERVAL};
use echlub_musician_sim::synth::{render_part, steady_beats};
use echlub_musician_sim::Role;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BotMode {
    Probe,
    Music { role: Role, bpm: f64 },
}

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub peer_id: u8,
    pub peers: usize,
    pub relay: SocketAddr,
    pub link: LinkProfile,
    /// Jitter depth per sender, indexed by peer id; the relay's mix stream
    /// uses this bot's own entry.
    pub jitter_depths: Vec<usize>,
    pub mode: BotMode,
    pub duration: Duration,
    pub seed: u64,
    pub record: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BotResult {
    pub peer_id: u8,
    pub frames_sent: u64,
    pub frames_dropped_uplink: u64,
    pub mixes_received: u64,
    pub mixes_dropped_downlink: u64,
    pub late_ticks: u64,
    pub jitter: JitterStats,
    /// `(emitter peer, one-way latency ms)` for each detected probe.
    pub detections: Vec<(usize, f64)>,
    #[serde(skip)]
    pub recording: Vec<i16>,
}

struct Scheduled {
    at: Instant,
    payload: Vec<u8>,
}

impl PartialEq for Scheduled {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}
impl Eq for Scheduled {}
impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> Ordering {
        other.at.cmp(&self.at)
    }
}

fn after_ms(t: Instant, ms: f64) -> Instant {
    t + Duration::from_secs_f64(ms.max(0.0) / 1000.0)
}

fn ms_since(epoch: Instant, t: Instant) -> f64 {
    t.duration_since(epoch).as_secs_f64() * 1000.0
}

fn music_source(role: Role, bpm: f64, duration: Duration) -> Vec<i16> {
    let beats = (duration.as_secs_f64() * bpm / 60.0).ceil() as usize + 1;
    let mut buf =
        vec![0f32; (duration.as_secs_f64() * SAMPLE_RATE as f64) as usize + FRAME_SAMPLES];
    render_part(&mut buf, role, &steady_beats(bpm, beats), 0.0);
    // Headroom so a four-way mix rarely clips.
    buf.iter()
        .map(|s| (s.clamp(-1.5, 1.5) * 0.15 * i16::MAX as f32) as i16)
        .collect()
}

/// Run one bot to completion. `epoch` must be shared by all bots in a run.
pub fn run_bot(config: BotConfig, epoch: Instant) -> io::Result<BotResult> {
    let bind: SocketAddr = if config.relay.ip().is_loopback() {
        "127.0.0.1:0".parse().expect("addr")
    } else {
        "0.0.0.0:0".parse().expect("addr")
    };
    let socket = UdpSocket::bind(bind)?;
    socket.connect(config.relay)?;
    socket.set_nonblocking(true)?;
    socket.send(
        &Packet::Hello {
            peer_id: config.peer_id,
        }
        .encode(),
    )?;

    let me = config.peer_id as usize;
    let probe = ProbeSchedule::new(config.peers);
    let source = match config.mode {
        BotMode::Music { role, bpm } => Some(music_source(role, bpm, config.duration)),
        BotMode::Probe => None,
    };
    let mut up = ImpairedLink::new(config.link.clone(), config.seed.wrapping_mul(2));
    let mut down = ImpairedLink::new(config.link.clone(), config.seed.wrapping_mul(2) + 1);
    // One buffer per sender: the relay's mix id in mix mode, each peer in forward mode.
    let mut buffers: BTreeMap<u8, JitterBuffer> = BTreeMap::new();
    // Probe clicks must not be smeared by concealment; music gets the softer fill.
    let concealment = match config.mode {
        BotMode::Probe => Concealment::Silence,
        BotMode::Music { .. } => Concealment::RepeatFade,
    };
    let mut detector = OnsetDetector::new(4_000, 50.0);
    let mut outbox: BinaryHeap<Scheduled> = BinaryHeap::new();
    let mut inbox: BinaryHeap<Scheduled> = BinaryHeap::new();
    let mut result = BotResult {
        peer_id: config.peer_id,
        ..Default::default()
    };
    let period = Duration::from_secs_f64(frame_ms() / 1000.0);
    let total_ticks = (config.duration.as_secs_f64() / period.as_secs_f64()) as u64;
    let mut buf = [0u8; 2048];
    let mut tick: u64 = 1;

    while tick <= total_ticks {
        let deadline = epoch + period * tick as u32;
        let now = Instant::now();
        while outbox.peek().is_some_and(|s| s.at <= now) {
            let s = outbox.pop().expect("peeked");
            let _ = socket.send(&s.payload);
        }
        while inbox.peek().is_some_and(|s| s.at <= now) {
            let s = inbox.pop().expect("peeked");
            if let Ok(Packet::Audio(f)) = Packet::decode(&s.payload) {
                buffers
                    .entry(f.peer_id)
                    .or_insert_with(|| {
                        let depth = config
                            .jitter_depths
                            .get(f.peer_id as usize)
                            .unwrap_or(&config.jitter_depths[me]);
                        JitterBuffer::new(*depth, FRAME_SAMPLES).with_concealment(concealment)
                    })
                    .push(f.seq, f.samples);
            }
        }
        if now >= deadline {
            if now - deadline > period {
                result.late_ticks += 1;
            }
            // Playout: the frame that starts at this tick.
            let mut heard = vec![0i16; FRAME_SAMPLES];
            for jb in buffers.values_mut() {
                for (acc, s) in heard.iter_mut().zip(jb.pop().1) {
                    *acc = acc.saturating_add(s);
                }
            }
            let start_ms = ms_since(epoch, deadline);
            for onset in detector.process(&heard, start_ms) {
                if let Some((src, emit)) = probe.attribute(onset, me) {
                    result.detections.push((src, onset - emit));
                }
            }
            let first = (tick - 1) * FRAME_SAMPLES as u64;
            let captured: Vec<i16> = match &source {
                Some(src) => {
                    let from = first as usize;
                    src.get(from..from + FRAME_SAMPLES)
                        .map(<[i16]>::to_vec)
                        .unwrap_or_else(|| vec![0; FRAME_SAMPLES])
                }
                None => probe.render(me, first, FRAME_SAMPLES),
            };
            if config.record {
                // Local monitoring: hear yourself immediately plus the mix.
                let own_now = match &source {
                    Some(src) => src
                        .get(first as usize + FRAME_SAMPLES..first as usize + 2 * FRAME_SAMPLES)
                        .map(<[i16]>::to_vec)
                        .unwrap_or_else(|| vec![0; FRAME_SAMPLES]),
                    None => vec![0; FRAME_SAMPLES],
                };
                result
                    .recording
                    .extend(heard.iter().zip(own_now).map(|(a, b)| a.saturating_add(b)));
            }
            let packet = Packet::Audio(AudioFrame {
                peer_id: config.peer_id,
                seq: tick as u32,
                sample_time: first,
                samples: captured,
            })
            .encode();
            match up.transit_ms() {
                Some(t) if t <= 0.0 => {
                    let _ = socket.send(&packet);
                    result.frames_sent += 1;
                }
                Some(t) => {
                    outbox.push(Scheduled {
                        at: after_ms(now, t),
                        payload: packet,
                    });
                    result.frames_sent += 1;
                }
                None => result.frames_dropped_uplink += 1,
            }
            tick += 1;
            continue;
        }

        let mut wake = deadline;
        for q in [&outbox, &inbox] {
            if let Some(s) = q.peek() {
                wake = wake.min(s.at);
            }
        }
        loop {
            match socket.recv(&mut buf) {
                Ok(len) => {
                    result.mixes_received += 1;
                    match down.transit_ms() {
                        Some(t) => inbox.push(Scheduled {
                            at: after_ms(Instant::now(), t),
                            payload: buf[..len].to_vec(),
                        }),
                        None => result.mixes_dropped_downlink += 1,
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                // Relay not up yet or just left.
                Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => break,
                Err(e) => return Err(e),
            }
        }
        // Socket read timeouts are too coarse on some platforms to clock audio.
        thread::sleep(wake.saturating_duration_since(now).min(POLL_INTERVAL));
    }
    let _ = socket.send(
        &Packet::Bye {
            peer_id: config.peer_id,
        }
        .encode(),
    );
    result.jitter = buffers
        .values()
        .map(JitterBuffer::stats)
        .fold(JitterStats::default(), JitterStats::merge);
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct BandConfig {
    pub relay: Option<SocketAddr>,
    /// Only used for an in-process relay.
    pub forward: bool,
    pub relay_depth: usize,
    pub links: Vec<LinkProfile>,
    /// `jitter_depths[listener][source]`.
    pub jitter_depths: Vec<Vec<usize>>,
    pub modes: Vec<BotMode>,
    pub duration: Duration,
    pub record: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BandResult {
    pub bots: Vec<BotResult>,
    pub relay: Option<RelayStats>,
}

/// Run a band of bots, spawning an in-process relay when `relay` is `None`.
pub fn run_band(config: BandConfig) -> io::Result<BandResult> {
    let stop = Arc::new(AtomicBool::new(false));
    let (relay_addr, relay_handle) = match config.relay {
        Some(addr) => (addr, None),
        None => {
            let socket = UdpSocket::bind("127.0.0.1:0")?;
            let addr = socket.local_addr()?;
            let stop = Arc::clone(&stop);
            let relay_config = RelayConfig {
                forward: config.forward,
                jitter_depth: config.relay_depth,
                max_peers: config.modes.len(),
                ..Default::default()
            };
            let handle = thread::spawn(move || echlub_jam_relay::run(socket, relay_config, &stop));
            (addr, Some(handle))
        }
    };
    // Give the relay a moment and align bots on a common future epoch.
    let epoch = Instant::now() + Duration::from_millis(50);
    let peers = config.modes.len();
    let handles: Vec<_> = config
        .modes
        .iter()
        .enumerate()
        .map(|(i, mode)| {
            let bot = BotConfig {
                peer_id: i as u8,
                peers,
                relay: relay_addr,
                link: config.links[i].clone(),
                jitter_depths: config.jitter_depths[i].clone(),
                mode: *mode,
                duration: config.duration,
                seed: 1 + i as u64,
                record: config.record,
            };
            thread::spawn(move || {
                thread::sleep(epoch.saturating_duration_since(Instant::now()));
                run_bot(bot, epoch)
            })
        })
        .collect();
    let mut bots = Vec::new();
    for h in handles {
        bots.push(
            h.join()
                .map_err(|_| io::Error::other("bot thread panicked"))??,
        );
    }
    stop.store(true, AtomicOrdering::Relaxed);
    let relay = match relay_handle {
        Some(h) => Some(
            h.join()
                .map_err(|_| io::Error::other("relay thread panicked"))??,
        ),
        None => None,
    };
    Ok(BandResult { bots, relay })
}
