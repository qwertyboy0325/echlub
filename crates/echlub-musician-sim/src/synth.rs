//! Tiny procedural synthesizer that renders each role's rock part from beat
//! onset times, so simulated performances can be listened to.
//!
//! Pattern: 4/4, eighth-note feel, I-V-vi-IV in E (E, B, C#m, A) per bar.

use echlub_jam::SAMPLE_RATE;

use crate::musician::Role;

const SR: f64 = SAMPLE_RATE as f64;
/// Bass roots per bar (Hz): E1, B1, C#2, A1.
const ROOTS: [f64; 4] = [41.20, 61.74, 69.30, 55.00];
/// Vocal melody (Hz), one note per beat across the 4-bar progression.
const MELODY: [f64; 16] = [
    329.63, 329.63, 369.99, 415.30, 369.99, 369.99, 311.13, 293.66, 277.18, 329.63, 277.18, 246.94,
    277.18, 246.94, 220.00, 246.94,
];

fn noise(n: u64, salt: u64) -> f64 {
    let mut z = n.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 52) as f64 - 1.0
}

/// Add one voice starting at `start_ms` (shifted by `shift_ms`) into `buf`.
fn add_voice(buf: &mut [f32], start_ms: f64, dur_ms: f64, voice: impl Fn(f64, u64) -> f64) {
    if start_ms < 0.0 {
        return;
    }
    let start = (start_ms * SR / 1000.0) as usize;
    let len = (dur_ms * SR / 1000.0) as usize;
    for i in 0..len {
        let Some(slot) = buf.get_mut(start + i) else {
            break;
        };
        *slot += voice(i as f64 / SR, (start + i) as u64) as f32;
    }
}

/// Eighth-note onsets interpolated between consecutive beats.
fn eighths(beats: &[f64]) -> Vec<(usize, f64, f64)> {
    let mut out = Vec::new();
    for k in 0..beats.len() {
        let next = beats.get(k + 1).copied().unwrap_or(
            beats[k]
                + if k > 0 {
                    beats[k] - beats[k - 1]
                } else {
                    500.0
                },
        );
        let half = (next - beats[k]) / 2.0;
        out.push((k * 2, beats[k], half));
        out.push((k * 2 + 1, beats[k] + half, half));
    }
    out
}

/// Render `role`'s part into `buf` from beat onsets (ms), delayed by `shift_ms`.
pub fn render_part(buf: &mut [f32], role: Role, beats: &[f64], shift_ms: f64) {
    let tau = std::f64::consts::TAU;
    for (e, t, len) in eighths(beats) {
        let t = t + shift_ms;
        let bar = e / 8;
        let pos = e % 8;
        let root = ROOTS[bar % 4];
        match role {
            Role::Drums => {
                add_voice(buf, t, 60.0, |s, n| 0.18 * noise(n, 1) * (-s * 90.0).exp());
                if matches!(pos, 0 | 4 | 5) {
                    add_voice(buf, t, 250.0, |s, _| {
                        let f = 50.0 + 100.0 * (-s * 30.0).exp();
                        0.9 * (tau * f * s).sin() * (-s * 18.0).exp()
                    });
                }
                if matches!(pos, 2 | 6) {
                    add_voice(buf, t, 200.0, |s, n| {
                        (0.5 * noise(n, 2) + 0.3 * (tau * 190.0 * s).sin()) * (-s * 22.0).exp()
                    });
                }
            }
            Role::Bass => add_voice(buf, t, len * 0.9, |s, _| {
                let f = root * 2.0;
                0.45 * ((tau * f * s).sin() + 0.4 * (tau * 2.0 * f * s).sin()) * (-s * 4.0).exp()
            }),
            Role::Guitar => add_voice(buf, t, len * 0.85, |s, _| {
                let saw = |f: f64| 2.0 * ((f * s).fract()) - 1.0;
                let f = root * 4.0;
                0.12 * (saw(f) + saw(f * 1.5) + saw(f * 2.0)) * (-s * 9.0).exp()
            }),
            Role::Vocals => {
                if pos % 2 == 0 {
                    let f = MELODY[(e / 2) % MELODY.len()];
                    add_voice(buf, t, len * 1.8, |s, _| {
                        let vib = 1.0 + 0.006 * (tau * 5.5 * s).sin();
                        let env = (s / 0.03).min(1.0) * (-s * 2.0).exp();
                        0.3 * env
                            * ((tau * f * vib * s).sin() + 0.3 * (tau * 2.0 * f * vib * s).sin())
                    });
                }
            }
        }
    }
}

/// What `listener` hears: its own part immediately, everyone else delayed by
/// `latency_ms[from][listener]`. Returns normalized 16-bit samples.
pub fn render_listener_mix(
    roles: &[Role],
    onsets_ms: &[Vec<f64>],
    latency_ms: &[Vec<f64>],
    listener: usize,
    tail_ms: f64,
) -> Vec<i16> {
    let end_ms = onsets_ms
        .iter()
        .filter_map(|o| o.last())
        .fold(0.0f64, |a, b| a.max(*b))
        + latency_ms.iter().flatten().fold(0.0f64, |a, b| a.max(*b))
        + tail_ms;
    let mut buf = vec![0f32; (end_ms * SR / 1000.0) as usize];
    for (j, role) in roles.iter().enumerate() {
        let shift = if j == listener {
            0.0
        } else {
            latency_ms[j][listener]
        };
        render_part(&mut buf, *role, &onsets_ms[j], shift);
    }
    let peak = buf.iter().fold(0f32, |a, b| a.max(b.abs())).max(1e-6);
    let gain = 0.9 / peak;
    buf.iter()
        .map(|s| (s * gain * i16::MAX as f32) as i16)
        .collect()
}

/// A steady performance at `bpm` for `beats` beats (no human timing).
pub fn steady_beats(bpm: f64, beats: usize) -> Vec<f64> {
    (0..beats).map(|k| k as f64 * 60_000.0 / bpm).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_are_audible_and_bounded() {
        let beats = steady_beats(120.0, 8);
        for role in Role::ROCK_FOUR_PIECE {
            let mut buf = vec![0f32; 48_000 * 5];
            render_part(&mut buf, role, &beats, 0.0);
            let peak = buf.iter().fold(0f32, |a, b| a.max(b.abs()));
            assert!(peak > 0.05 && peak < 3.0, "{role:?} peak {peak}");
        }
    }

    #[test]
    fn delay_shifts_onset() {
        let beats = steady_beats(120.0, 4);
        let first = |shift: f64| {
            let mut buf = vec![0f32; 48_000 * 3];
            render_part(&mut buf, Role::Drums, &beats, shift);
            buf.iter().position(|s| s.abs() > 1e-4).unwrap()
        };
        assert_eq!(first(30.0) - first(0.0), 1440);
    }

    #[test]
    fn listener_mix_is_normalized() {
        let roles = Role::ROCK_FOUR_PIECE;
        let onsets: Vec<_> = roles.iter().map(|_| steady_beats(120.0, 8)).collect();
        let lat = crate::ensemble::uniform_matrix(4, 25.0);
        let mix = render_listener_mix(&roles, &onsets, &lat, 0, 500.0);
        let peak = mix.iter().map(|s| s.unsigned_abs()).max().unwrap();
        assert!(peak > 25_000);
    }
}
