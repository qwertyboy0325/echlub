//! Ensemble timing simulation: N musicians, each hearing the others through
//! a latency matrix, playing a fixed number of bars of 4/4 rock.

use echlub_jam::rng::SplitMix64;
use serde::Serialize;

use crate::musician::{MusicianParams, Role};

/// Verdict thresholds. These are hypotheses to be calibrated against real
/// musicians, not established perceptual limits.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Thresholds {
    pub tight_drift_pct: f64,
    pub tight_heard_rms_ms: f64,
    pub playable_drift_pct: f64,
    pub playable_heard_rms_ms: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            tight_drift_pct: 2.0,
            tight_heard_rms_ms: 30.0,
            playable_drift_pct: 5.0,
            playable_heard_rms_ms: 45.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Tight,
    Playable,
    Struggling,
}

#[derive(Debug, Clone)]
pub struct EnsembleConfig {
    pub names: Vec<String>,
    pub players: Vec<MusicianParams>,
    /// `latency_ms[from][to]`: mouth-to-ear delay with which `to` hears `from`.
    pub latency_ms: Vec<Vec<f64>>,
    pub bpm: f64,
    pub bars: usize,
    pub seed: u64,
    pub thresholds: Thresholds,
}

impl EnsembleConfig {
    /// Rock four-piece with the same latency on every path.
    pub fn rock_uniform(latency_ms: f64, bpm: f64, bars: usize, seed: u64) -> Self {
        let roles = Role::ROCK_FOUR_PIECE;
        let n = roles.len();
        Self {
            names: roles.iter().map(|r| r.name().to_string()).collect(),
            players: roles
                .iter()
                .map(|r| MusicianParams::rock_default(*r))
                .collect(),
            latency_ms: uniform_matrix(n, latency_ms),
            bpm,
            bars,
            seed,
            thresholds: Thresholds::default(),
        }
    }
}

pub fn uniform_matrix(n: usize, latency_ms: f64) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| if i == j { 0.0 } else { latency_ms })
                .collect()
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerResult {
    pub name: String,
    pub role: Role,
    /// RMS of (heard other onset - own onset) over all beats and partners.
    pub heard_rms_async_ms: f64,
    /// Mean of the same; positive means others sound late to this player.
    pub heard_mean_async_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnsembleReport {
    pub evidence_level: &'static str,
    pub target_bpm: f64,
    pub start_bpm: f64,
    pub end_bpm: f64,
    /// (end_bpm - target_bpm) / target_bpm * 100.
    pub tempo_drift_pct: f64,
    /// Worst player's heard RMS asynchrony.
    pub worst_heard_rms_ms: f64,
    /// Mean per-beat SD of actual onsets across players (as at a neutral point).
    pub onset_spread_sd_ms: f64,
    /// True when some path is so slow that a beat cannot be heard before the
    /// next one is planned; results are then outside the model's validity.
    pub causality_warning: bool,
    pub verdict: Verdict,
    pub thresholds: Thresholds,
    pub players: Vec<PlayerResult>,
    #[serde(skip)]
    pub onsets_ms: Vec<Vec<f64>>,
}

pub fn simulate(config: &EnsembleConfig) -> EnsembleReport {
    let n = config.players.len();
    assert!(n >= 1 && config.latency_ms.len() == n && config.names.len() == n);
    let beats = (config.bars * 4).max(16);
    let t0 = 60_000.0 / config.bpm;
    let l = &config.latency_ms;
    let leader = config
        .players
        .iter()
        .position(|p| p.role == Role::Drums)
        .unwrap_or(0);

    let mut rngs: Vec<_> = (0..n)
        .map(|i| SplitMix64::new(config.seed.wrapping_mul(1_000_003).wrapping_add(i as u64)))
        .collect();
    let mut planned = vec![vec![0.0; beats]; n];
    let mut actual = vec![vec![0.0; beats]; n];
    let mut period = vec![t0; n];

    // Count-in: the drummer clicks four; everyone else enters on what they heard.
    for i in 0..n {
        let p = &config.players[i];
        planned[i][0] = if i == leader {
            0.0
        } else {
            l[leader][i] * (1.0 - p.latency_compensation)
        };
        actual[i][0] = planned[i][0] + p.motor_sd_ms * rngs[i].normal();
    }

    let weight_sum = |i: usize| -> f64 {
        (0..n)
            .filter(|j| *j != i)
            .map(|j| config.players[j].role.listen_weight())
            .sum()
    };

    let mut causality_warning = false;
    for k in 0..beats - 1 {
        for i in 0..n {
            let p = &config.players[i];
            let asynchrony = if n > 1 {
                let ws = weight_sum(i);
                let mut heard = 0.0;
                let mut lat = 0.0;
                for j in (0..n).filter(|j| *j != i) {
                    let w = config.players[j].role.listen_weight();
                    heard += w * (actual[j][k] + l[j][i]);
                    lat += w * l[j][i];
                    if l[j][i] > period[i] / 2.0 {
                        causality_warning = true;
                    }
                }
                actual[i][k] - (heard / ws - p.latency_compensation * lat / ws) + p.anticipation_ms
            } else {
                0.0
            };
            planned[i][k + 1] = planned[i][k] + period[i] + p.timekeeper_sd_ms * rngs[i].normal()
                - p.alpha * asynchrony;
            actual[i][k + 1] = planned[i][k + 1] + p.motor_sd_ms * rngs[i].normal();
            period[i] = (period[i] - p.beta * asynchrony + p.gamma * (t0 - period[i]))
                .clamp(t0 * 0.5, t0 * 2.0);
        }
    }

    report(config, actual, causality_warning)
}

fn report(
    config: &EnsembleConfig,
    actual: Vec<Vec<f64>>,
    causality_warning: bool,
) -> EnsembleReport {
    let n = actual.len();
    let beats = actual[0].len();
    let mean_onset: Vec<f64> = (0..beats)
        .map(|k| actual.iter().map(|a| a[k]).sum::<f64>() / n as f64)
        .collect();
    let window = 8.min(beats / 2);
    let bpm_over = |from: usize| {
        let ioi = (mean_onset[from + window] - mean_onset[from]) / window as f64;
        60_000.0 / ioi
    };
    // Skip the first bar while players settle after the count-in.
    let start_bpm = bpm_over(4.min(beats - window - 1));
    let end_bpm = bpm_over(beats - window - 1);

    let players: Vec<PlayerResult> = (0..n)
        .map(|i| {
            let mut sum = 0.0;
            let mut sq = 0.0;
            let mut count = 0.0;
            let settled = 4.min(beats - 1);
            for (k, own) in actual[i].iter().enumerate().skip(settled) {
                for j in (0..n).filter(|j| *j != i) {
                    let e = actual[j][k] + config.latency_ms[j][i] - own;
                    sum += e;
                    sq += e * e;
                    count += 1.0;
                }
            }
            let count: f64 = if count > 0.0 { count } else { 1.0 };
            PlayerResult {
                name: config.names[i].clone(),
                role: config.players[i].role,
                heard_rms_async_ms: (sq / count).sqrt(),
                heard_mean_async_ms: sum / count,
            }
        })
        .collect();

    let onset_spread_sd_ms = (0..beats)
        .map(|k| {
            let m = mean_onset[k];
            (actual.iter().map(|a| (a[k] - m).powi(2)).sum::<f64>() / n as f64).sqrt()
        })
        .sum::<f64>()
        / beats as f64;

    let worst_heard_rms_ms = players
        .iter()
        .map(|p| p.heard_rms_async_ms)
        .fold(0.0, f64::max);
    let tempo_drift_pct = (end_bpm - config.bpm) / config.bpm * 100.0;
    let th = config.thresholds;
    let verdict = if causality_warning {
        Verdict::Struggling
    } else if tempo_drift_pct.abs() <= th.tight_drift_pct
        && worst_heard_rms_ms <= th.tight_heard_rms_ms
    {
        Verdict::Tight
    } else if tempo_drift_pct.abs() <= th.playable_drift_pct
        && worst_heard_rms_ms <= th.playable_heard_rms_ms
    {
        Verdict::Playable
    } else {
        Verdict::Struggling
    };

    EnsembleReport {
        evidence_level: "Simulation",
        target_bpm: config.bpm,
        start_bpm,
        end_bpm,
        tempo_drift_pct,
        worst_heard_rms_ms,
        onset_spread_sd_ms,
        causality_warning,
        verdict,
        thresholds: th,
        players,
        onsets_ms: actual,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SweepRow {
    pub latency_ms: f64,
    pub runs: usize,
    pub mean_tempo_drift_pct: f64,
    pub mean_worst_heard_rms_ms: f64,
    pub tight_fraction: f64,
    pub playable_or_better_fraction: f64,
}

/// Rock four-piece at each uniform latency, averaged over `runs` seeds.
pub fn sweep_uniform_latency(
    latencies_ms: &[f64],
    bpm: f64,
    bars: usize,
    runs: usize,
    compensation: f64,
) -> Vec<SweepRow> {
    latencies_ms
        .iter()
        .map(|&lat| {
            let reports: Vec<_> = (0..runs as u64)
                .map(|seed| {
                    let mut cfg = EnsembleConfig::rock_uniform(lat, bpm, bars, seed);
                    for p in &mut cfg.players {
                        p.latency_compensation = compensation;
                    }
                    simulate(&cfg)
                })
                .collect();
            let r = runs.max(1) as f64;
            SweepRow {
                latency_ms: lat,
                runs,
                mean_tempo_drift_pct: reports.iter().map(|x| x.tempo_drift_pct).sum::<f64>() / r,
                mean_worst_heard_rms_ms: reports.iter().map(|x| x.worst_heard_rms_ms).sum::<f64>()
                    / r,
                tight_fraction: reports
                    .iter()
                    .filter(|x| x.verdict == Verdict::Tight)
                    .count() as f64
                    / r,
                playable_or_better_fraction: reports
                    .iter()
                    .filter(|x| x.verdict != Verdict::Struggling)
                    .count() as f64
                    / r,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_seed() {
        let cfg = EnsembleConfig::rock_uniform(20.0, 120.0, 32, 7);
        let a = simulate(&cfg);
        let b = simulate(&cfg);
        assert_eq!(a.onsets_ms, b.onsets_ms);
    }

    #[test]
    fn zero_latency_band_stays_on_tempo() {
        let rows = sweep_uniform_latency(&[0.0], 120.0, 32, 20, 0.0);
        let row = &rows[0];
        assert!(row.mean_tempo_drift_pct.abs() < 2.0, "{row:?}");
        assert!(row.tight_fraction >= 0.8, "{row:?}");
    }

    #[test]
    fn latency_drags_tempo_down() {
        let rows = sweep_uniform_latency(&[0.0, 60.0], 120.0, 32, 20, 0.0);
        assert!(
            rows[1].mean_tempo_drift_pct < rows[0].mean_tempo_drift_pct - 3.0,
            "{rows:?}"
        );
        assert!(rows[1].playable_or_better_fraction < rows[0].playable_or_better_fraction);
    }

    #[test]
    fn compensation_reduces_drag() {
        let naive = sweep_uniform_latency(&[40.0], 120.0, 32, 20, 0.0);
        let comp = sweep_uniform_latency(&[40.0], 120.0, 32, 20, 0.8);
        assert!(comp[0].mean_tempo_drift_pct.abs() < naive[0].mean_tempo_drift_pct.abs());
    }

    #[test]
    fn anticipation_absorbs_small_latency() {
        let rows = sweep_uniform_latency(&[10.0], 120.0, 32, 20, 0.0);
        assert!(rows[0].mean_tempo_drift_pct.abs() < 1.0, "{rows:?}");
    }

    #[test]
    fn huge_latency_flags_causality() {
        let r = simulate(&EnsembleConfig::rock_uniform(400.0, 120.0, 16, 1));
        assert!(r.causality_warning);
        assert_eq!(r.verdict, Verdict::Struggling);
    }
}
