use hanabi_protocol::{HanabiLiveSessionState, HanabiLiveSnapshot};
use serde_json::json;

#[test]
fn recorded_no_variant_session_matches_snapshot_rebuild_and_redacts_own_hand() {
    // Exact opening draws formerly embedded in CLI traced_opening_snapshot.
    // No recorded strategic choice is asserted by this protocol contract.
    let mut snapshot: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/no-variant-live-opening.json")).unwrap();
    let mut session = HanabiLiveSessionState::new();
    let (table, initial) = session
        .apply_json(&json!({"kind": "initialize", "snapshot": snapshot}).to_string())
        .unwrap();
    assert_eq!(table, 39);
    assert_eq!(initial.hands.len(), 3);
    assert!(initial.hands[0].iter().all(|card| card.identity.is_none()));
    let turn = json!({"type": "turn", "num": 0, "currentPlayerIndex": 0});
    snapshot["actions"]
        .as_array_mut()
        .unwrap()
        .push(turn.clone());
    let append = json!({"kind": "append", "tableID": table, "actions": [turn]});
    let (_, incremental) = session.apply_json(&append.to_string()).unwrap();
    let rebuilt = HanabiLiveSnapshot::from_json(&snapshot.to_string())
        .unwrap()
        .player_view()
        .unwrap();
    assert_eq!(incremental, rebuilt);
    assert_eq!(initial, rebuilt);
    // A failed transport update must preserve the usable session.
    assert!(
        session
            .apply_json(
                &json!({
                    "kind": "append", "tableID": table + 1, "actions": []
                })
                .to_string()
            )
            .is_err()
    );
    assert_eq!(session.apply_json(&append.to_string()).unwrap().1, rebuilt);
}
