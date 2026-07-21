pub mod derived;
pub mod privacy;
pub mod schema;
pub mod summarize;
pub mod validate;

pub use derived::compute_derived_metrics;
pub use schema::{
    EvidenceLevel, EvidenceStatus, MetricValue, PerformanceRunV1, RunMetadata,
    SyntheticPulseMetrics, TimingMetrics, TransportObservation, SCHEMA_VERSION,
};
pub use summarize::{summarize_run, SummaryReport};
pub use validate::{parse_and_validate, validate_run, ValidationError, ValidationResult};
