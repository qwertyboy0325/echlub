//! Timing probes: each peer emits a click in its own slot of a repeating
//! cycle; listeners detect clicks in what they hear and attribute them back
//! to the emitter to measure one-way latency in the signal itself.

use crate::SAMPLE_RATE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProbeSchedule {
    pub cycle_ms: f64,
    pub peers: usize,
    pub pulse_samples: usize,
    pub amplitude: i16,
}

impl ProbeSchedule {
    pub fn new(peers: usize) -> Self {
        Self {
            cycle_ms: 1000.0,
            peers: peers.max(1),
            pulse_samples: 48,
            amplitude: 12_000,
        }
    }

    pub fn slot_ms(&self) -> f64 {
        self.cycle_ms / self.peers as f64
    }

    /// Emission time of `peer`'s click in `cycle`, in ms.
    pub fn emit_ms(&self, peer: usize, cycle: u64) -> f64 {
        cycle as f64 * self.cycle_ms + peer as f64 * self.slot_ms() + 5.0
    }

    fn emit_sample(&self, peer: usize, cycle: u64) -> u64 {
        (self.emit_ms(peer, cycle) * SAMPLE_RATE as f64 / 1000.0).round() as u64
    }

    /// Render `len` samples of `peer`'s probe signal starting at `start`.
    pub fn render(&self, peer: usize, start: u64, len: usize) -> Vec<i16> {
        let cycle_samples = (self.cycle_ms * SAMPLE_RATE as f64 / 1000.0) as u64;
        (0..len as u64)
            .map(|i| {
                let n = start + i;
                let cycle = n / cycle_samples;
                let begin = self.emit_sample(peer, cycle);
                if n >= begin && n < begin + self.pulse_samples as u64 {
                    // 2 kHz square burst; first sample is full scale.
                    if ((n - begin) / 12).is_multiple_of(2) {
                        self.amplitude
                    } else {
                        -self.amplitude
                    }
                } else {
                    0
                }
            })
            .collect()
    }

    /// Map a detection back to `(emitter, emit_ms)`: the latest emission by
    /// another peer at or before the detection, within one cycle.
    pub fn attribute(&self, detect_ms: f64, listener: usize) -> Option<(usize, f64)> {
        let cycle = (detect_ms / self.cycle_ms).floor().max(0.0) as u64;
        let mut best: Option<(usize, f64)> = None;
        for c in cycle.saturating_sub(1)..=cycle {
            for peer in (0..self.peers).filter(|p| *p != listener) {
                let e = self.emit_ms(peer, c);
                if e <= detect_ms + 1e-9 && best.is_none_or(|(_, b)| e > b) {
                    best = Some((peer, e));
                }
            }
        }
        best.filter(|(_, e)| detect_ms - e < self.cycle_ms)
    }
}

/// Threshold onset detector with a refractory period.
#[derive(Debug, Clone)]
pub struct OnsetDetector {
    threshold: i16,
    refractory_ms: f64,
    last_onset_ms: Option<f64>,
}

impl OnsetDetector {
    pub fn new(threshold: i16, refractory_ms: f64) -> Self {
        Self {
            threshold,
            refractory_ms,
            last_onset_ms: None,
        }
    }

    /// Returns onset times (ms) found in `frame`, which starts at `start_ms`.
    pub fn process(&mut self, frame: &[i16], start_ms: f64) -> Vec<f64> {
        let mut onsets = Vec::new();
        for (i, s) in frame.iter().enumerate() {
            if s.unsigned_abs() < self.threshold.unsigned_abs() {
                continue;
            }
            let t = start_ms + i as f64 * 1000.0 / SAMPLE_RATE as f64;
            if self
                .last_onset_ms
                .is_none_or(|last| t - last >= self.refractory_ms)
            {
                onsets.push(t);
                self.last_onset_ms = Some(t);
            }
        }
        onsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_and_detect_at_emit_time() {
        let sched = ProbeSchedule::new(2);
        let signal = sched.render(1, 0, 48_000);
        let mut det = OnsetDetector::new(4_000, 50.0);
        let onsets = det.process(&signal, 0.0);
        assert_eq!(onsets.len(), 1);
        assert!((onsets[0] - sched.emit_ms(1, 0)).abs() < 0.03);
    }

    #[test]
    fn attribution_picks_latest_other_peer() {
        let sched = ProbeSchedule::new(4);
        // Peer 2 emits at 505 ms; heard 30 ms later by peer 0.
        assert_eq!(sched.attribute(535.0, 0), Some((2, 505.0)));
        // Listener's own slot is skipped.
        assert_eq!(sched.attribute(535.0, 2), Some((1, 255.0)));
    }
}
