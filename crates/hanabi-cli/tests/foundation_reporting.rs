//! Physical report invariants for the recorded foundation bug reproduction.
use std::{path::PathBuf, process::Command};

#[test]
fn white_analysis_reports_preserve_visible_faces_and_hide_own_hand() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../hanabi-search/src/h_group/tests/fixtures/white-seed1-foundation.json");
    let output = Command::new(env!("CARGO_BIN_EXE_hanabi-engine"))
        .arg("analyze")
        .arg(fixture)
        .args([
            "--live-turn",
            "46",
            "--format",
            "json",
            "--convention",
            "none",
            "--exact-world-limit",
            "1",
            "--exact-node-limit",
            "1",
            "--move-time-ms",
            "1000",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["board"]["stacks"].as_array().unwrap().len(), 6);
    let hands = report["board"]["hands"].as_array().unwrap();
    assert!(
        hands[0]["cards"]
            .as_array()
            .unwrap()
            .iter()
            .all(|card| card["visibleIdentity"].is_null())
    );
    let white_two = hands[1]["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|card| card["card"] == 40)
        .unwrap();
    // Exact physical source data, not an inference or a preferred-move oracle.
    assert_eq!(white_two["visibleIdentity"], "s5:2");
    let link = report["replayLink"].as_str().unwrap();
    assert_eq!(link.split(',').nth(2).unwrap().replace('-', ""), "13");
}
