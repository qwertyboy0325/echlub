use echlub_model::{CanonicalDocumentV1, DocumentState, StateHash};

pub fn normalized_hash(state: &DocumentState) -> StateHash {
    let canonical = CanonicalDocumentV1::from_state(state);
    let bytes = postcard::to_allocvec(&canonical).expect("canonical serialization");
    StateHash(*blake3::hash(&bytes).as_bytes())
}
