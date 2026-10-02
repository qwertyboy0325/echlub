//! Deterministic simulation of the star topology at frame granularity.
//!
//! Peers capture 128-sample frames, send them over impaired uplinks to a
//! relay that jitter-buffers each peer, mixes minus-one, and sends mixes over
//! impaired downlinks to client jitter buffers. Probe clicks are detected in
//! the played-out signal, so network-path latency, concealment, and buffer
//! behaviour are measured from audio rather than computed. Device latency
//! (capture/playback buffers, converters, codec framing) is added from the
//! analytic budget because there is no audio hardware in the loop.
//!
//! All clocks are ideal and synchronized; clock drift is not modelled.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use serde::Serialize;

use crate::budget::{device_ms, forward_depth, star_path, PeerSetup, Topology};
use crate::jitter::{JitterBuffer, JitterStats};
use crate::mixer::mix_minus;
use crate::netsim::ImpairedLink;
use crate::probe::{OnsetDetector, ProbeSchedule};
use crate::{frame_ms, FRAME_SAMPLES};

/// Probes emitted closer than this to the end are not expected to arrive.
const TAIL_MARGIN_MS: f64 = 300.0;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub peers: Vec<PeerSetup>,
    pub duration_ms: f64,
    pub seed: u64,
    pub topology: Topology,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LatencyStats {
    pub count: usize,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
}

impl LatencyStats {
    pub fn from_samples(values: &[f64]) -> Option<LatencyStats> {
        if values.is_empty() {
            return None;
        }
        let mut v = values.to_vec();
        v.sort_by(f64::total_cmp);
        let pct = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
        Some(LatencyStats {
            count: v.len(),
            mean_ms: v.iter().sum::<f64>() / v.len() as f64,
            p50_ms: pct(0.5),
            p95_ms: pct(0.95),
            max_ms: *v.last().expect("non-empty"),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PairResult {
    pub from: String,
    pub to: String,
    pub expected_probes: u64,
    pub detected_probes: u64,
    /// Measured in simulation: capture instant to playout instant.
    pub network_path: Option<LatencyStats>,
    /// Analytic device-side latency added on top.
    pub device_ms: f64,
    pub mouth_to_ear_p50_ms: Option<f64>,
    pub mouth_to_ear_p95_ms: Option<f64>,
    /// Analytic budget for comparison with the measured value.
    pub analytic_budget_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineReport {
    pub evidence_level: &'static str,
    pub note: &'static str,
    pub frame_ms: f64,
    pub duration_ms: f64,
    pub seed: u64,
    pub topology: Topology,
    pub peers: Vec<PeerSetup>,
    pub pairs: Vec<PairResult>,
    pub relay_jitter: Vec<JitterStats>,
    pub client_jitter: Vec<JitterStats>,
}

impl PipelineReport {
    /// `m[from][to]` mouth-to-ear p50 latency (diagonal is 0: local monitoring).
    pub fn latency_matrix_p50(&self) -> Vec<Vec<f64>> {
        self.matrix(|p| p.mouth_to_ear_p50_ms)
    }

    pub fn latency_matrix_p95(&self) -> Vec<Vec<f64>> {
        self.matrix(|p| p.mouth_to_ear_p95_ms)
    }

    fn matrix(&self, pick: impl Fn(&PairResult) -> Option<f64>) -> Vec<Vec<f64>> {
        let n = self.peers.len();
        let mut m = vec![vec![0.0; n]; n];
        for (idx, pair) in self.pairs.iter().enumerate() {
            let (from, to) = pair_indices(n, idx);
            m[from][to] = pick(pair).unwrap_or(pair.analytic_budget_ms);
        }
        m
    }
}

fn pair_indices(n: usize, idx: usize) -> (usize, usize) {
    let from = idx / (n - 1);
    let k = idx % (n - 1);
    let to = if k >= from { k + 1 } else { k };
    (from, to)
}

struct Delivery {
    at: f64,
    target: usize,
    /// Source slot within the target's buffers (always 0 for mixes).
    slot: usize,
    seq: u32,
    samples: Vec<i16>,
}

impl PartialEq for Delivery {
    fn eq(&self, other: &Self) -> bool {
        self.at.total_cmp(&other.at) == Ordering::Equal
    }
}
impl Eq for Delivery {}
impl PartialOrd for Delivery {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Delivery {
    // Reversed: BinaryHeap pops the earliest arrival first.
    fn cmp(&self, other: &Self) -> Ordering {
        other.at.total_cmp(&self.at)
    }
}

fn drain(queue: &mut BinaryHeap<Delivery>, until: f64, buffers: &mut [Vec<JitterBuffer>]) {
    while queue.peek().is_some_and(|d| d.at <= until) {
        let d = queue.pop().expect("peeked");
        buffers[d.target][d.slot].push(d.seq, d.samples);
    }
}

fn sum_stats(stats: impl Iterator<Item = JitterStats>) -> JitterStats {
    stats.fold(JitterStats::default(), |a, b| JitterStats {
        received: a.received + b.received,
        played: a.played + b.played,
        concealed: a.concealed + b.concealed,
        late_dropped: a.late_dropped + b.late_dropped,
        overflow_dropped: a.overflow_dropped + b.overflow_dropped,
        duplicates: a.duplicates + b.duplicates,
        resets: a.resets + b.resets,
    })
}

pub fn run(config: &PipelineConfig) -> PipelineReport {
    let n = config.peers.len();
    assert!(n >= 2, "pipeline needs at least two peers");
    let p = frame_ms();
    let ticks = (config.duration_ms / p).floor() as u64;
    let probe = ProbeSchedule::new(n);

    let seed = |i: usize, dir: u64| {
        config
            .seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(i as u64 * 2 + dir)
    };
    let mut up: Vec<_> = (0..n)
        .map(|i| ImpairedLink::new(config.peers[i].link.clone(), seed(i, 0)))
        .collect();
    let mut down: Vec<_> = (0..n)
        .map(|i| ImpairedLink::new(config.peers[i].link.clone(), seed(i, 1)))
        .collect();
    let forward = config.topology == Topology::Forward;
    let peers = &config.peers;
    let mut relay_jb: Vec<Vec<JitterBuffer>> = if forward {
        Vec::new()
    } else {
        peers
            .iter()
            .map(|p| vec![JitterBuffer::new(p.jitter_depth, FRAME_SAMPLES)])
            .collect()
    };
    // Mix: one buffer per client. Forward: one per (client, source).
    let mut client_jb: Vec<Vec<JitterBuffer>> = (0..n)
        .map(|d| {
            if forward {
                (0..n)
                    .map(|s| {
                        let depth = if s == d {
                            1
                        } else {
                            forward_depth(&peers[s], &peers[d])
                        };
                        JitterBuffer::new(depth, FRAME_SAMPLES)
                    })
                    .collect()
            } else {
                vec![JitterBuffer::new(peers[d].jitter_depth, FRAME_SAMPLES)]
            }
        })
        .collect();
    let mut detectors = vec![OnsetDetector::new(4_000, 50.0); n];
    let mut latencies = vec![vec![Vec::<f64>::new(); n]; n];
    let mut up_q = BinaryHeap::new();
    let mut down_q = BinaryHeap::new();

    for k in 0..ticks {
        let t0 = k as f64 * p;
        let seq = k as u32;

        let tc = t0 + 0.25 * p;
        drain(&mut down_q, tc, &mut client_jb);
        for (j, buffers) in client_jb.iter_mut().enumerate() {
            let mut frame = vec![0i16; FRAME_SAMPLES];
            for (slot, jb) in buffers.iter_mut().enumerate() {
                if forward && slot == j {
                    continue;
                }
                for (acc, s) in frame.iter_mut().zip(jb.pop().1) {
                    *acc = acc.saturating_add(s);
                }
            }
            for onset in detectors[j].process(&frame, tc) {
                match probe.attribute(onset, j) {
                    Some((src, emit)) if emit + TAIL_MARGIN_MS < config.duration_ms => {
                        latencies[src][j].push(onset - emit);
                    }
                    _ => {}
                }
            }
        }

        if !forward {
            let tr = t0 + 0.5 * p;
            drain(&mut up_q, tr, &mut relay_jb);
            let inputs: Vec<Vec<i16>> = relay_jb.iter_mut().map(|jb| jb[0].pop().1).collect();
            for (j, mix) in mix_minus(&inputs, FRAME_SAMPLES).into_iter().enumerate() {
                if let Some(transit) = down[j].transit_ms() {
                    down_q.push(Delivery {
                        at: tr + transit,
                        target: j,
                        slot: 0,
                        seq,
                        samples: mix,
                    });
                }
            }
        }

        let ts = t0 + p;
        for (i, link) in up.iter_mut().enumerate() {
            let samples = probe.render(i, k * FRAME_SAMPLES as u64, FRAME_SAMPLES);
            let Some(transit) = link.transit_ms() else {
                continue;
            };
            if forward {
                // Relay forwards on arrival; each listener gets its own copy.
                for j in (0..n).filter(|j| *j != i) {
                    if let Some(back) = down[j].transit_ms() {
                        down_q.push(Delivery {
                            at: ts + transit + back,
                            target: j,
                            slot: i,
                            seq,
                            samples: samples.clone(),
                        });
                    }
                }
            } else {
                up_q.push(Delivery {
                    at: ts + transit,
                    target: i,
                    slot: 0,
                    seq,
                    samples,
                });
            }
        }
    }

    let mut pairs = Vec::new();
    for (from, heard_from) in latencies.iter().enumerate() {
        let expected = (0..)
            .take_while(|c| probe.emit_ms(from, *c) + TAIL_MARGIN_MS < config.duration_ms)
            .count() as u64;
        for to in (0..n).filter(|t| *t != from) {
            let src = &config.peers[from];
            let dst = &config.peers[to];
            let stats = LatencyStats::from_samples(&heard_from[to]);
            let dev = device_ms(src, dst);
            pairs.push(PairResult {
                from: src.name.clone(),
                to: dst.name.clone(),
                expected_probes: expected,
                detected_probes: heard_from[to].len() as u64,
                mouth_to_ear_p50_ms: stats.as_ref().map(|s| s.p50_ms + dev),
                mouth_to_ear_p95_ms: stats.as_ref().map(|s| s.p95_ms + dev),
                network_path: stats,
                device_ms: dev,
                analytic_budget_ms: star_path(src, dst, config.topology).total_ms,
            });
        }
    }

    PipelineReport {
        evidence_level: "Simulation",
        note: "Simulated pipeline with ASSUMED link/endpoint profiles. Not a measurement of any real network or device.",
        frame_ms: p,
        duration_ms: config.duration_ms,
        seed: config.seed,
        topology: config.topology,
        peers: config.peers.clone(),
        pairs,
        relay_jitter: relay_jb.iter().map(|j| j[0].stats()).collect(),
        client_jitter: client_jb
            .iter()
            .map(|b| sum_stats(b.iter().map(JitterBuffer::stats)))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::netsim::LinkProfile;
    use crate::profiles::EndpointProfile;

    fn peer(name: &str, link: LinkProfile) -> PeerSetup {
        PeerSetup::auto(
            name,
            link,
            EndpointProfile::preset("native-interface").unwrap(),
        )
    }

    #[test]
    fn pair_index_mapping() {
        let n = 3;
        let got: Vec<_> = (0..6).map(|i| pair_indices(n, i)).collect();
        assert_eq!(got, vec![(0, 1), (0, 2), (1, 0), (1, 2), (2, 0), (2, 1)]);
    }

    #[test]
    fn ideal_links_detect_every_probe_with_buffer_latency_only() {
        let report = run(&PipelineConfig {
            peers: vec![
                peer("a", LinkProfile::ideal()),
                peer("b", LinkProfile::ideal()),
            ],
            duration_ms: 5_000.0,
            seed: 1,
            topology: Topology::Mix,
        });
        for pair in &report.pairs {
            assert_eq!(pair.detected_probes, pair.expected_probes, "{pair:?}");
            let net = pair.network_path.as_ref().unwrap();
            // packetization + relay/client buffering only; a few frames.
            assert!(net.max_ms < 6.0 * frame_ms(), "{net:?}");
            assert!(net.p95_ms - net.p50_ms < 1e-6, "ideal path must not jitter");
        }
    }

    #[test]
    fn base_delay_shows_up_in_measurement() {
        let slow = LinkProfile::new("slow", 10.0, 0.0, 0.0);
        let fast = run(&PipelineConfig {
            peers: vec![
                peer("a", LinkProfile::ideal()),
                peer("b", LinkProfile::ideal()),
            ],
            duration_ms: 3_000.0,
            seed: 1,
            topology: Topology::Mix,
        });
        let delayed = run(&PipelineConfig {
            peers: vec![peer("a", slow.clone()), peer("b", slow)],
            duration_ms: 3_000.0,
            seed: 1,
            topology: Topology::Mix,
        });
        let f = fast.pairs[0].network_path.as_ref().unwrap().p50_ms;
        let d = delayed.pairs[0].network_path.as_ref().unwrap().p50_ms;
        // 10 ms up + 10 ms down, give or take one frame of tick alignment.
        assert!((d - f - 20.0).abs() <= frame_ms() + 1e-6, "{f} vs {d}");
    }

    #[test]
    fn deterministic_for_seed() {
        let cfg = PipelineConfig {
            peers: vec![
                peer("a", LinkProfile::new("j", 2.0, 2.0, 0.01)),
                peer("b", LinkProfile::new("j", 3.0, 1.0, 0.01)),
                peer("c", LinkProfile::new("j", 1.0, 3.0, 0.01)),
            ],
            duration_ms: 4_000.0,
            seed: 42,
            topology: Topology::Mix,
        };
        assert_eq!(run(&cfg).pairs, run(&cfg).pairs);
    }

    #[test]
    fn lossy_links_conceal_frames() {
        let lossy = LinkProfile::new("lossy", 2.0, 0.5, 0.05);
        let report = run(&PipelineConfig {
            peers: vec![peer("a", lossy.clone()), peer("b", lossy)],
            duration_ms: 4_000.0,
            seed: 5,
            topology: Topology::Mix,
        });
        assert!(report.client_jitter.iter().any(|s| s.concealed > 0));
    }

    #[test]
    fn forward_topology_is_faster_and_detects_all_peers() {
        let link = LinkProfile::new("j", 3.0, 0.6, 0.0);
        let cfg = |topology| PipelineConfig {
            peers: vec![
                peer("a", link.clone()),
                peer("b", link.clone()),
                peer("c", link.clone()),
            ],
            duration_ms: 5_000.0,
            seed: 3,
            topology,
        };
        let mix = run(&cfg(Topology::Mix));
        let fwd = run(&cfg(Topology::Forward));
        assert!(fwd.relay_jitter.is_empty());
        for (m, f) in mix.pairs.iter().zip(&fwd.pairs) {
            assert_eq!(f.detected_probes, f.expected_probes, "{f:?}");
            let (m, f) = (
                m.mouth_to_ear_p50_ms.unwrap(),
                f.mouth_to_ear_p50_ms.unwrap(),
            );
            assert!(f < m, "forward {f} should beat mix {m}");
        }
    }
}
