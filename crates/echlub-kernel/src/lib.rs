mod apply;
mod hash;
mod validate;

use echlub_model::{DocumentState, OperationEnvelope, StateHash, ValidationDecision};

pub use apply::{ApplyError, ApplyReceipt};
pub use validate::KernelError;

pub struct Kernel;

impl Kernel {
    pub fn validate(state: &DocumentState, envelope: &OperationEnvelope) -> ValidationDecision {
        validate::validate(state, envelope)
    }

    pub fn apply(
        state: &mut DocumentState,
        envelope: &OperationEnvelope,
    ) -> Result<ApplyReceipt, ApplyError> {
        apply::apply(state, envelope)
    }

    pub fn normalized_hash(state: &DocumentState) -> StateHash {
        hash::normalized_hash(state)
    }
}
