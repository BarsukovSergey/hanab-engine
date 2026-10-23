//! Opt-in White H-Group pipeline experiment; no replacement policy or live enablement.
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufWriter, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use hanabi_core::{
    Clue, FullState, GenericClue, ObservedEvent, PhysicalAction, PlayerView, VariantId, VariantSpec,
};
use hanabi_search::{
    HGroupLevel, HGroupProfile, InformationSet, PlannerConfig, PlannerResult, PlanningObjective,
    SupportedConvention, plan_move,
};

fn variant() -> VariantSpec {
    VariantSpec::from_catalog(VariantId::White6Suits)
}

fn deck(seed: u64) -> Vec<hanabi_core::Card> {
    let mut cards: Vec<_> = variant()
        .identities()
        .identities()
        .iter()
        .flat_map(|id| {
            std::iter::repeat_n(
                hanabi_core::Card::new(id.suit, id.rank),
                usize::from(id.copies),
            )
        })
        .collect();
    // splitmix64-fisher-yates-modulo-v1, identical to the physical product pilot.
    let mut rng = seed;
    for i in (1..cards.len()).rev() {
        rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        cards.swap(
            i,
            usize::try_from(z % u64::try_from(i + 1).unwrap()).unwrap(),
        );
    }
    cards
}

fn config() -> PlannerConfig {
    PlannerConfig {
        objective: PlanningObjective::ExpectedScore,
        exact_world_limit: 16,
        exact_node_limit: 256,
        move_time_limit: Duration::from_secs(5),
    }
}

// Only an acting observer's redacted view crosses the decision boundary.
fn decide(view: PlayerView<GenericClue>, level: HGroupLevel) -> Result<PlannerResult, String> {
    assert_eq!(view.variant, variant());
    assert_eq!(view.observer, view.current_player);
    assert!(
        view.hands[view.observer.index()]
            .iter()
            .all(|card| card.identity.is_none())
    );
    for entry in &view.history {
        if let ObservedEvent::Drew {
            player, identity, ..
        } = &entry.event
        {
            assert!(
                *player != view.observer || identity.is_none(),
                "own draw identity leaked"
            );
        }
    }
    let legal = view.legal_physical_actions();
    let standard = view.try_standard().map_err(|e| e.to_string())?;
    assert_eq!(standard.variant, variant());
    let information =
        InformationSet::new(&standard).map_err(|e| format!("InformationSet::new: {e}"))?;
    let result = plan_move(
        &information,
        SupportedConvention::HGroup(HGroupProfile::Level(level)),
        config(),
    )
    .map_err(|e| format!("plan_move: {e}"))?;
    assert!(
        legal.contains(&PhysicalAction::from(result.best_action)),
        "illegal planner action"
    );
    Ok(result)
}

#[test]
fn contract_white_checked_labels_and_deck() {
    use std::collections::BTreeMap;
    let cards = deck(1);
    assert_eq!(cards.len(), 60);
    assert_eq!(cards, deck(1));
    assert_ne!(cards, deck(2));
    let mut counts = BTreeMap::new();
    for card in &cards {
        *counts.entry(*card).or_insert(0_usize) += 1;
    }
    for id in variant().identities().identities() {
        assert_eq!(
            counts.remove(&hanabi_core::Card::new(id.suit, id.rank)),
            Some(usize::from(id.copies))
        );
    }
    assert!(counts.is_empty());
    let state = FullState::new_variant(3, variant(), cards).unwrap();
    let view = state.physical_view_for(state.current_player()).unwrap();
    let standard = view.clone().try_standard().unwrap();
    assert_eq!(standard.variant, variant());
    assert_eq!(
        standard
            .legal_actions()
            .into_iter()
            .map(PhysicalAction::from)
            .collect::<Vec<_>>(),
        view.legal_physical_actions()
    );
    let spec = variant();
    for clue in spec
        .clue_colors()
        .iter()
        .copied()
        .map(GenericClue::Color)
        .chain(spec.clue_ranks().iter().copied().map(GenericClue::Rank))
    {
        let legacy = Clue::try_from(clue).unwrap();
        for id in spec.identities().identities() {
            assert_eq!(
                legacy.matches(hanabi_core::Card::new(id.suit, id.rank)),
                spec.touches(id.id, clue)
            );
            if id.suit.value() == 5 && matches!(clue, GenericClue::Color(_)) {
                assert!(
                    !spec.touches(id.id, clue),
                    "white has negative evidence for every color"
                );
            }
        }
    }
}

#[test]
#[ignore = "offline H-Group first-action smoke; explicit invocation only"]
fn white_first_choice() {
    let mut state = FullState::new_variant(3, variant(), deck(1)).unwrap();
    let started = Instant::now();
    let result = catch_unwind(AssertUnwindSafe(|| {
        decide(
            state.physical_view_for(state.current_player()).unwrap(),
            HGroupLevel::Level1,
        )
    }));
    let result = result.unwrap_or_else(|payload| Err(panic_message(&*payload)));
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/product-pilot/white-hgroup-first.json");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    let cards: Vec<_> = deck(1).iter().enumerate().map(|(id, card)| json!({"cardId": id, "suit": card.suit.value(), "rank": card.rank.value()})).collect();
    fs::write(
        output,
        serde_json::to_vec_pretty(
            &json!({"codeSha": String::from_utf8(sha.stdout).unwrap().trim(),
        "profile": "H-Group Level1", "seed": 1, "orderedDeck": cards,
        "config": format!("{:?}", config()), "moveSeconds": started.elapsed().as_secs_f64(),
        "result": format!("{result:?}")}),
        )
        .unwrap(),
    )
    .unwrap();
    state
        .apply_physical(PhysicalAction::from(result.unwrap().best_action))
        .unwrap();
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    let detail = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic");
    format!("decision panic: {detail}")
}

fn write_line(writer: &mut impl Write, value: &Value) {
    serde_json::to_writer(&mut *writer, value).unwrap();
    writeln!(writer).unwrap();
    writer.flush().unwrap();
}

fn action_json(action: PhysicalAction) -> Value {
    match action {
        PhysicalAction::Play(id) => json!({"kind": "Play", "cardId": id.index()}),
        PhysicalAction::Discard(id) => json!({"kind": "Discard", "cardId": id.index()}),
        PhysicalAction::Clue { target, clue } => {
            json!({"kind": "Clue", "target": target.index(), "clue": match clue {
                GenericClue::Color(color) => json!({"Color": color.value()}),
                GenericClue::Rank(rank) => json!({"Rank": rank.value()}),
            }})
        }
    }
}

#[test]
#[ignore = "offline White Level4 game; explicit invocation only"]
// Keep the flushed failure evidence and exact reconstruction beside the game loop.
#[allow(clippy::too_many_lines)]
fn white_hgroup_game() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/product-pilot/white-hgroup.json");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    let sha = String::from_utf8(sha.stdout).unwrap();
    let seed = 1;
    let cards = deck(seed);
    let ordered_deck: Vec<_> = cards.iter().enumerate().map(|(id, card)|
        json!({"cardId": id, "suit": card.suit.value(), "rank": card.rank.value()})).collect();
    let mut trace = BufWriter::new(fs::File::create(output.with_extension("ndjson")).unwrap());
    write_line(
        &mut trace,
        &json!({"event": "initial", "seed": seed, "codeSha": sha.trim(),
        "shuffle": "splitmix64-fisher-yates-modulo-v1", "orderedDeck": ordered_deck,
        "profile": "H-Group Level4", "config": format!("{:?}", config()),
        "semanticStatus": "White 3/4 Loaded Play Clue Assumption not yet verified; no comprehensive White support claim"}),
    );
    let mut state = FullState::new_variant(3, variant(), cards.clone()).unwrap();
    let started = Instant::now();
    let mut actions = Vec::new();
    let mut decisions = Vec::new();
    let mut error = None;
    while !state.is_terminal() && state.turn() < 200 {
        write_line(
            &mut trace,
            &json!({"event": "beforeDecision", "turn": state.turn(), "actor": state.current_player().index()}),
        );
        let move_started = Instant::now();
        let result = catch_unwind(AssertUnwindSafe(|| {
            decide(
                state.physical_view_for(state.current_player()).unwrap(),
                HGroupLevel::Level4,
            )
        }));
        let seconds = move_started.elapsed().as_secs_f64();
        let result = match result {
            Ok(Ok(result)) => result,
            Ok(Err(cause)) => {
                write_line(
                    &mut trace,
                    &json!({"event": "decisionError", "turn": state.turn(), "moveSeconds": seconds, "error": cause}),
                );
                error = Some(cause);
                break;
            }
            Err(payload) => {
                let cause = panic_message(&*payload);
                write_line(
                    &mut trace,
                    &json!({"event": "decisionError", "turn": state.turn(), "moveSeconds": seconds, "error": cause}),
                );
                error = Some(cause);
                break;
            }
        };
        let action = PhysicalAction::from(result.best_action);
        let decision = json!({"turn": state.turn(), "actor": state.current_player().index(),
            "action": action_json(action), "moveSeconds": seconds,
            "phase": format!("{:?}", result.phase), "exactStatus": format!("{:?}", result.exact_status),
            "budgetExhausted": result.budget_exhausted,
            "rootActions": format!("{:?}", result.root_actions),
            "semanticAssumptions": "existing Level4 inference; White-specific meanings not independently certified"});
        write_line(
            &mut trace,
            &json!({"event": "selected", "decision": decision}),
        );
        if let Err(cause) = state.apply_physical(action) {
            error = Some(format!("apply_physical: {cause}"));
            break;
        }
        actions.push(action);
        decisions.push(decision);
        write_line(
            &mut trace,
            &json!({"event": "applied", "turn": state.turn(), "stackScore": state.score(), "strikes": state.strikes()}),
        );
    }
    if !state.is_terminal() && error.is_none() {
        error = Some("INCOMPLETE: 200-turn cap".into());
    }
    let mut replay = FullState::new_variant(3, variant(), cards).unwrap();
    for action in &actions {
        replay.apply_physical(*action).unwrap();
    }
    let reconstruction_matches = replay == state;
    let counts = actions.iter().fold([0_usize; 3], |mut counts, action| {
        counts[match action {
            PhysicalAction::Play(_) => 0,
            PhysicalAction::Discard(_) => 1,
            PhysicalAction::Clue { .. } => 2,
        }] += 1;
        counts
    });
    let report = json!({"codeSha": sha.trim(), "seed": seed, "orderedDeck": ordered_deck,
        "variant": "White6Suits", "players": 3, "profile": "H-Group Level4", "config": format!("{:?}", config()),
        "shuffle": "splitmix64-fisher-yates-modulo-v1", "completed": state.is_terminal(),
        "terminalReason": format!("{:?}", state.status()), "officialFinalScore": state.final_score(),
        "stackScore": state.score(), "strikes": state.strikes(), "turns": state.turn(),
        "actionCounts": {"Play": counts[0], "Discard": counts[1], "Clue": counts[2]},
        "warnings": ["White rank-3/4 Loaded Play Clue Assumption is not implemented by this pilot; White semantics are not certified"],
        "elapsedSeconds": started.elapsed().as_secs_f64(), "error": error,
        "observerRedactionCheckedEveryDecision": true, "reconstructionMatches": reconstruction_matches,
        "decisions": decisions});
    write_line(&mut trace, &json!({"event": "result", "report": report}));
    fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    assert!(
        error.is_none() && reconstruction_matches && state.is_terminal(),
        "INCOMPLETE: inspect white-hgroup.json"
    );
    assert!(
        state.final_score().is_some_and(|score| score > 0),
        "no positive official score"
    );
}
