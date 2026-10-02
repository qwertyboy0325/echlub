//! Network impairment model: base one-way delay, exponential jitter tail,
//! independent loss.

use serde::Serialize;

use crate::rng::SplitMix64;

/// Where a number came from. Simulation inputs stay `Assumed` until replaced
/// by field measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Provenance {
    Assumed,
    Measured,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LinkProfile {
    pub name: String,
    /// Minimum one-way delay in ms.
    pub base_ms: f64,
    /// Mean of the exponential queueing delay added on top of `base_ms`.
    pub jitter_mean_ms: f64,
    /// Independent packet loss probability.
    pub loss: f64,
    pub provenance: Provenance,
}

impl LinkProfile {
    pub fn new(name: impl Into<String>, base_ms: f64, jitter_mean_ms: f64, loss: f64) -> Self {
        Self {
            name: name.into(),
            base_ms,
            jitter_mean_ms,
            loss,
            provenance: Provenance::Assumed,
        }
    }

    pub fn ideal() -> Self {
        Self::new("ideal", 0.0, 0.0, 0.0)
    }

    pub fn expected_ms(&self) -> f64 {
        self.base_ms + self.jitter_mean_ms
    }

    /// 99th percentile of the exponential jitter component.
    pub fn p99_jitter_ms(&self) -> f64 {
        self.jitter_quantile_ms(0.99)
    }

    /// Jitter that a buffer must absorb so that a `coverage` fraction of
    /// packets arrives in time (exponential tail).
    pub fn jitter_quantile_ms(&self, coverage: f64) -> f64 {
        -self.jitter_mean_ms * (1.0 - coverage.clamp(0.0, 0.999_999)).ln()
    }

    /// Two segments in series. Jitter means add (approximation).
    pub fn then(&self, next: &LinkProfile) -> LinkProfile {
        let provenance =
            if self.provenance == Provenance::Measured && next.provenance == Provenance::Measured {
                Provenance::Measured
            } else {
                Provenance::Assumed
            };
        LinkProfile {
            name: format!("{}+{}", self.name, next.name),
            base_ms: self.base_ms + next.base_ms,
            jitter_mean_ms: self.jitter_mean_ms + next.jitter_mean_ms,
            loss: 1.0 - (1.0 - self.loss) * (1.0 - next.loss),
            provenance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImpairedLink {
    pub profile: LinkProfile,
    rng: SplitMix64,
}

impl ImpairedLink {
    pub fn new(profile: LinkProfile, seed: u64) -> Self {
        Self {
            profile,
            rng: SplitMix64::new(seed),
        }
    }

    /// One-way transit time for the next packet, or `None` if it is lost.
    pub fn transit_ms(&mut self) -> Option<f64> {
        if self.rng.next_f64() < self.profile.loss {
            return None;
        }
        Some(self.profile.base_ms + self.rng.exponential(self.profile.jitter_mean_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_link_is_instant_and_lossless() {
        let mut l = ImpairedLink::new(LinkProfile::ideal(), 1);
        for _ in 0..100 {
            assert_eq!(l.transit_ms(), Some(0.0));
        }
    }

    #[test]
    fn loss_rate_is_respected() {
        let mut l = ImpairedLink::new(LinkProfile::new("lossy", 1.0, 0.5, 0.1), 3);
        let n = 20_000;
        let lost = (0..n).filter(|_| l.transit_ms().is_none()).count();
        let rate = lost as f64 / n as f64;
        assert!((rate - 0.1).abs() < 0.01, "rate {rate}");
    }

    #[test]
    fn transit_never_below_base() {
        let mut l = ImpairedLink::new(LinkProfile::new("j", 3.0, 2.0, 0.0), 9);
        assert!((0..1000).all(|_| l.transit_ms().unwrap() >= 3.0));
    }

    #[test]
    fn series_combination() {
        let a = LinkProfile::new("a", 1.0, 0.5, 0.1);
        let b = LinkProfile::new("b", 2.0, 1.0, 0.1);
        let c = a.then(&b);
        assert_eq!(c.base_ms, 3.0);
        assert_eq!(c.jitter_mean_ms, 1.5);
        assert!((c.loss - 0.19).abs() < 1e-12);
        assert_eq!(c.provenance, Provenance::Assumed);
    }
}
