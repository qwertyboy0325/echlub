//! Fixed-depth jitter buffer with silence concealment.
//!
//! Playout starts once `target_depth` frames are queued; each [`pop`] then
//! returns the next sequence number or a concealed (silent) frame. Late
//! packets are dropped, excess queueing (clock drift, bursts) is trimmed, and
//! a long silence resets the buffer so a returning peer re-buffers cleanly.
//!
//! [`pop`]: JitterBuffer::pop

use std::collections::BTreeMap;

use serde::Serialize;

/// Consecutive empty pops after which playout stops and re-buffers.
const RESET_AFTER_EMPTY: u32 = 64;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct JitterStats {
    pub received: u64,
    pub played: u64,
    pub concealed: u64,
    pub late_dropped: u64,
    pub overflow_dropped: u64,
    pub duplicates: u64,
    pub resets: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopKind {
    /// Not yet started; output is silence and nothing was expected.
    Buffering,
    Played,
    Concealed,
}

#[derive(Debug, Clone)]
pub struct JitterBuffer {
    target_depth: usize,
    max_depth: usize,
    frame_len: usize,
    frames: BTreeMap<u32, Vec<i16>>,
    next_seq: Option<u32>,
    started: bool,
    empty_streak: u32,
    stats: JitterStats,
}

impl JitterBuffer {
    pub fn new(target_depth: usize, frame_len: usize) -> Self {
        let target_depth = target_depth.max(1);
        Self {
            target_depth,
            max_depth: target_depth * 2 + 4,
            frame_len,
            frames: BTreeMap::new(),
            next_seq: None,
            started: false,
            empty_streak: 0,
            stats: JitterStats::default(),
        }
    }

    pub fn target_depth(&self) -> usize {
        self.target_depth
    }

    pub fn stats(&self) -> JitterStats {
        self.stats
    }

    pub fn queued(&self) -> usize {
        self.frames.len()
    }

    pub fn push(&mut self, seq: u32, samples: Vec<i16>) {
        if self.started && self.next_seq.is_some_and(|next| seq < next) {
            self.stats.late_dropped += 1;
            return;
        }
        if self.frames.contains_key(&seq) {
            self.stats.duplicates += 1;
            return;
        }
        self.stats.received += 1;
        self.frames.insert(seq, samples);
        if self.started && self.frames.len() > self.max_depth {
            let last = *self.frames.keys().next_back().expect("non-empty");
            let keep_from = last.saturating_sub(self.target_depth as u32 - 1);
            let before = self.frames.len();
            self.frames = self.frames.split_off(&keep_from);
            self.stats.overflow_dropped += (before - self.frames.len()) as u64;
            self.next_seq = Some(keep_from);
        }
    }

    /// Next frame for playout. Always returns `frame_len` samples.
    pub fn pop(&mut self) -> (PopKind, Vec<i16>) {
        if !self.started {
            if self.frames.len() < self.target_depth {
                return (PopKind::Buffering, vec![0; self.frame_len]);
            }
            self.started = true;
            self.next_seq = self.frames.keys().next().copied();
            self.empty_streak = 0;
        }
        let next = self.next_seq.expect("started implies next_seq");
        self.next_seq = Some(next.wrapping_add(1));
        // Anything older than `next` can never play.
        while let Some((&k, _)) = self.frames.iter().next() {
            if k < next {
                self.frames.remove(&k);
                self.stats.late_dropped += 1;
            } else {
                break;
            }
        }
        match self.frames.remove(&next) {
            Some(mut f) => {
                f.resize(self.frame_len, 0);
                self.stats.played += 1;
                self.empty_streak = 0;
                (PopKind::Played, f)
            }
            None => {
                self.stats.concealed += 1;
                if self.frames.is_empty() {
                    self.empty_streak += 1;
                    if self.empty_streak >= RESET_AFTER_EMPTY {
                        self.started = false;
                        self.next_seq = None;
                        self.stats.resets += 1;
                    }
                }
                (PopKind::Concealed, vec![0; self.frame_len])
            }
        }
    }
}

/// Smallest depth (in frames) whose span covers `p99_jitter_ms` plus one
/// frame of tick-phase slack.
pub fn recommended_depth(p99_jitter_ms: f64, frame_ms: f64) -> usize {
    ((p99_jitter_ms / frame_ms).ceil() as usize + 1).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(v: i16) -> Vec<i16> {
        vec![v; 4]
    }

    #[test]
    fn buffers_then_plays_in_order() {
        let mut jb = JitterBuffer::new(2, 4);
        jb.push(10, frame(1));
        assert_eq!(jb.pop().0, PopKind::Buffering);
        jb.push(11, frame(2));
        assert_eq!(jb.pop(), (PopKind::Played, frame(1)));
        assert_eq!(jb.pop(), (PopKind::Played, frame(2)));
        assert_eq!(jb.pop().0, PopKind::Concealed);
    }

    #[test]
    fn reorders_and_conceals_gaps() {
        let mut jb = JitterBuffer::new(2, 4);
        jb.push(1, frame(1));
        jb.push(0, frame(0));
        jb.push(3, frame(3));
        assert_eq!(jb.pop().1, frame(0));
        assert_eq!(jb.pop().1, frame(1));
        assert_eq!(jb.pop(), (PopKind::Concealed, frame(0)));
        assert_eq!(jb.pop().1, frame(3));
        assert_eq!(jb.stats().concealed, 1);
    }

    #[test]
    fn drops_late_and_duplicate() {
        let mut jb = JitterBuffer::new(1, 4);
        jb.push(5, frame(5));
        jb.pop();
        jb.push(5, frame(5));
        jb.push(4, frame(4));
        jb.push(6, frame(6));
        jb.push(6, frame(6));
        let s = jb.stats();
        assert_eq!(s.late_dropped, 2);
        assert_eq!(s.duplicates, 1);
    }

    #[test]
    fn trims_overflow_to_target() {
        let mut jb = JitterBuffer::new(2, 4);
        jb.push(0, frame(0));
        jb.push(1, frame(1));
        jb.pop();
        for s in 2..20 {
            jb.push(s, frame(s as i16));
        }
        assert!(jb.queued() <= 2 * 2 + 4);
        assert!(jb.stats().overflow_dropped > 0);
        assert_eq!(jb.pop().0, PopKind::Played);
    }

    #[test]
    fn resets_after_long_silence() {
        let mut jb = JitterBuffer::new(1, 4);
        jb.push(0, frame(0));
        jb.pop();
        for _ in 0..RESET_AFTER_EMPTY {
            jb.pop();
        }
        assert_eq!(jb.stats().resets, 1);
        jb.push(500, frame(9));
        assert_eq!(jb.pop(), (PopKind::Played, frame(9)));
    }

    #[test]
    fn depth_recommendation() {
        assert_eq!(recommended_depth(0.0, 2.667), 1);
        assert_eq!(recommended_depth(5.0, 2.667), 3);
    }
}
