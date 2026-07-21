use echlub_web::EchlubCore;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_node_experimental);

#[wasm_bindgen_test]
fn wasm_facade_initialization() {
    let core = EchlubCore::new(1, 1);
    assert_eq!(core.state_hash_hex().len(), 64);
}

#[wasm_bindgen_test]
fn wasm_accepted_operation() {
    let mut core = EchlubCore::new(1, 1);
    let decision = core.add_track(1, "Piano");
    assert!(decision.contains("Accepted"));
}

#[wasm_bindgen_test]
fn wasm_rejected_operation() {
    let mut core = EchlubCore::new(1, 1);
    core.add_note(99, 1, 0, 480, 60, 100);
    assert!(core.last_decision_json().contains("MissingDependency"));
}
