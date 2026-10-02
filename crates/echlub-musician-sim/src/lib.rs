//! Simulated rock musicians for the real-time jam lab.
//!
//! Rule-based entities, not language models: each musician keeps time with a
//! linear phase/period-correction model reacting to what it *hears* (other
//! players delayed by the latency matrix), and can render its part as audio.
//! Model parameters are planning assumptions to be calibrated against real
//! players; outputs are simulations, not evidence about humans.

pub mod ensemble;
pub mod musician;
pub mod synth;

pub use ensemble::{simulate, sweep_uniform_latency, EnsembleConfig, EnsembleReport, Verdict};
pub use musician::{MusicianParams, Role};
