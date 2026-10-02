//! Platform-neutral real-time jam core.
//!
//! Packet format, jitter buffering, mix-minus, latency budgets, network
//! impairment models, and a deterministic transport-pipeline simulator.
//! No sockets, threads, audio devices, or browser APIs live here so the same
//! code can back a native client, a relay server, or a WASM AudioWorklet.

pub mod budget;
pub mod jitter;
pub mod mixer;
pub mod netsim;
pub mod packet;
pub mod pipeline;
pub mod probe;
pub mod profiles;
pub mod rng;
pub mod wav;

/// Prototype audio sample rate.
pub const SAMPLE_RATE: u32 = 48_000;

/// Samples per network frame (128 samples = 2.667 ms at 48 kHz).
pub const FRAME_SAMPLES: usize = 128;

/// Duration of one network frame in milliseconds.
pub fn frame_ms() -> f64 {
    FRAME_SAMPLES as f64 * 1000.0 / SAMPLE_RATE as f64
}

/// Convert a sample count to milliseconds at [`SAMPLE_RATE`].
pub fn samples_to_ms(samples: f64) -> f64 {
    samples * 1000.0 / SAMPLE_RATE as f64
}
