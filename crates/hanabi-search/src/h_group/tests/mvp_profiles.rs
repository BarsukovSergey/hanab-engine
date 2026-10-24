//! Bounded reachability on reviewed fixtures, not per-rule certification.
use super::*;
use std::time::{Duration, Instant};

#[test]
#[ignore = "opt-in bounded MVP profile checkpoint"]
fn contract_no_variant_mvp_profile_reachability() {
    let positions = [
        ("p4v0s9", expert_replay_p4v0s9(), 0),
        ("p4v0s9", expert_replay_p4v0s9(), 1),
        ("p4v0s2", expert_replay_p4v0s2(), 0),
    ];
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/no-variant-profiles.json");
    let started = Instant::now();
    let mut rows = (1..=26)
        .map(|level| serde_json::json!({"level": level, "status": "UNTESTED", "positions": []}))
        .collect::<Vec<_>>();
    let mut first_failure = None;
    for number in 1..=26_u8 {
        let profile = if number == 26 {
            HGroupProfile::Max
        } else {
            HGroupProfile::Level(HGroupLevel::try_from(number).unwrap())
        };
        let row = &mut rows[usize::from(number - 1)];
        let profile_started = Instant::now();
        for (seed, replay, turn) in &positions {
            let state = replay.state_at_turn(*turn).unwrap();
            let observer = state.current_player();
            let case_started = Instant::now();
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert!(
                    started.elapsed() < Duration::from_secs(60),
                    "total time bound"
                );
                let control = crate::AnalysisControl::default();
                let _budget = crate::budget::Scope::enter(Duration::from_secs(2), &control);
                crate::budget::run(|| {
                    let view = state.view_for(observer).unwrap();
                    assert!(
                        view.hands[observer.index()]
                            .iter()
                            .all(|card| card.identity.is_none())
                    );
                    let deductions = LogicalDeductions::new(view.clone()).unwrap();
                    let analysis = crate::SupportedConvention::HGroup(profile).analyze(&deductions);
                    assert!(!analysis.actions.is_empty(), "no admitted actions");
                    for candidate in &analysis.actions {
                        state
                            .clone()
                            .apply(candidate.action)
                            .expect("admitted action is legal");
                    }
                    if let Some(action) = analysis.forced_action {
                        assert!(
                            analysis
                                .actions
                                .iter()
                                .any(|candidate| candidate.action == action)
                        );
                        let mut after = state.clone();
                        after.apply(action).expect("required action is legal");
                        assert_eq!(after.strikes(), state.strikes(), "false required play");
                    }
                    // At the opening, permute only the observer's own hidden
                    // cards. Compare the complete analysis of equal legal views.
                    if *turn == 0 {
                        let mut deck = (0..50)
                            .map(|index| state.card(CardId::new(index)).unwrap())
                            .collect::<Vec<_>>();
                        let other_slot = (1..4).find(|&slot| deck[slot] != deck[0]).unwrap();
                        deck.swap(0, other_slot);
                        let alternative = FullState::new_standard(4, deck).unwrap();
                        let other_view = alternative.view_for(observer).unwrap();
                        assert_eq!(view, other_view);
                        let other = LogicalDeductions::new(other_view).unwrap();
                        assert_eq!(
                            analysis,
                            crate::SupportedConvention::HGroup(profile).analyze(&other)
                        );
                    }
                })
                .expect("case deadline reached");
            }));
            let mut cell = serde_json::json!({
                "seed": seed, "turn": turn + 1, "observer": observer.index(),
                "status": if outcome.is_ok() { "PASS" } else { "FAIL" },
                "elapsedSeconds": case_started.elapsed().as_secs_f64(),
            });
            if let Err(payload) = outcome {
                let reason = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("panic");
                cell["error"] = serde_json::json!(reason);
                first_failure = Some(cell.clone());
            }
            row["positions"].as_array_mut().unwrap().push(cell);
            if first_failure.is_some() {
                break;
            }
        }
        row["status"] = serde_json::json!(if first_failure.is_some() {
            "FAIL"
        } else {
            "PASS"
        });
        row["elapsedSeconds"] = serde_json::json!(profile_started.elapsed().as_secs_f64());
        let report = serde_json::json!({
            "scope": "Three reviewed No Variant opening positions; reachability only, not rule certification",
            "caseTimeSeconds": 2, "totalTimeSeconds": 60,
            "elapsedSeconds": started.elapsed().as_secs_f64(),
            "firstFailure": first_failure, "profiles": rows,
        });
        std::fs::write(
            &output,
            serde_json::to_string_pretty(&report).unwrap() + "\n",
        )
        .unwrap();
        if first_failure.is_some() {
            break;
        }
    }
    assert!(
        first_failure.is_none(),
        "profile checkpoint failed; inspect {}",
        output.display()
    );
}
