//! Opt-in physical lifecycle measurement, not a convention or strength contract.
use std::{
    env, fs,
    io::{BufWriter, Write},
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    process::Command,
    time::Instant,
};

use hanabi_core::{
    Card, FullState, GenericClue, ObservedEvent, PhysicalAction, PlayerView, VariantId, VariantSpec,
};
use hanabi_search::{ConventionAgnosticPolicy, LogicalDeductions};
use serde_json::{Value, json};

const SHUFFLE: &str = "splitmix64-fisher-yates-modulo-v1";

fn variant() -> VariantSpec {
    VariantSpec::from_catalog(VariantId::Rainbow6Suits)
}

fn deck(seed: u64) -> Vec<Card> {
    let mut cards: Vec<_> = variant()
        .identities()
        .identities()
        .iter()
        .flat_map(|id| std::iter::repeat_n(Card::new(id.suit, id.rank), usize::from(id.copies)))
        .collect();
    // SplitMix64 with wrapping arithmetic, descending Fisher-Yates. The small
    // modulo bias is intentional and versioned; this is not a statistical benchmark.
    let mut rng = seed;
    for i in (1..cards.len()).rev() {
        rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        let j = usize::try_from(z % u64::try_from(i + 1).unwrap()).unwrap();
        cards.swap(i, j);
    }
    cards
}

fn action_json(action: PhysicalAction) -> Value {
    match action {
        PhysicalAction::Play(card) => json!({"kind": "Play", "cardId": card.index()}),
        PhysicalAction::Discard(card) => json!({"kind": "Discard", "cardId": card.index()}),
        PhysicalAction::Clue { target, clue } => {
            let label = match clue {
                GenericClue::Color(color) => json!({"Color": color.value()}),
                GenericClue::Rank(rank) => json!({"Rank": rank.value()}),
            };
            json!({"kind": "Clue", "target": target.index(), "clue": label})
        }
    }
}

// This boundary accepts only the acting observation: no FullState, deck or seed.
fn decide(view: PlayerView<GenericClue>) -> Result<PhysicalAction, String> {
    if view.observer != view.current_player {
        return Err("observer is not current player".into());
    }
    if view.hands[view.observer.index()]
        .iter()
        .any(|card| card.identity.is_some())
    {
        return Err("own hand identity leaked".into());
    }
    for entry in &view.history {
        if let ObservedEvent::Drew {
            player, identity, ..
        } = &entry.event
        {
            if *player == view.observer && identity.is_some() {
                return Err("own draw history identity leaked".into());
            }
        }
    }
    let legal = view.legal_physical_actions();
    let deductions = LogicalDeductions::new(view).map_err(|error| error.to_string())?;
    let action = ConventionAgnosticPolicy
        .select_physical_action(&deductions)
        .map_err(|error| error.to_string())?;
    if !legal.contains(&action) {
        return Err(format!("illegal physical action: {action:?}"));
    }
    Ok(action)
}

fn write_line(writer: &mut impl Write, value: &Value) {
    serde_json::to_writer(&mut *writer, value).unwrap();
    writeln!(writer).unwrap();
    writer.flush().unwrap();
}

#[test]
fn deterministic_deck_and_reconstruction() {
    use std::collections::BTreeMap;
    let cards = deck(1);
    assert_eq!(cards, deck(1));
    assert_ne!(cards, deck(2));
    let mut counts = BTreeMap::new();
    for card in &cards {
        *counts.entry(*card).or_insert(0_usize) += 1;
    }
    for id in variant().identities().identities() {
        assert_eq!(
            counts.remove(&Card::new(id.suit, id.rank)),
            Some(usize::from(id.copies))
        );
    }
    assert!(counts.is_empty());
    let mut state = FullState::new_variant(3, variant(), cards.clone()).unwrap();
    let mut replay = FullState::new_variant(3, variant(), cards).unwrap();
    // Invariant-only prefix, never an assertion that a choice is strategically good.
    for _ in 0..2 {
        let action = decide(state.physical_view_for(state.current_player()).unwrap()).unwrap();
        state.apply_physical(action).unwrap();
        replay.apply_physical(action).unwrap();
        assert_eq!(state, replay);
    }
}

#[test]
#[ignore = "offline full-game product pilot; explicit release invocation only"]
// Keep checkpoint writes, decision isolation and failure reporting together for review.
#[allow(clippy::too_many_lines)]
fn rainbow_physical_pilot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join(
        env::var("HANABI_PRODUCT_PILOT_REPORT")
            .unwrap_or_else(|_| "target/product-pilot/rainbow.json".into()),
    );
    let seeds: Vec<u64> = env::var("HANABI_PRODUCT_PILOT_SEEDS")
        .unwrap_or_else(|_| "1".into())
        .split(',')
        .map(|seed| seed.parse().expect("integer seed"))
        .collect();
    assert!(
        !seeds.is_empty() && seeds.len() <= 3,
        "one to three fixed seeds only"
    );
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(sha.status.success());
    let sha = String::from_utf8(sha.stdout).unwrap().trim().to_owned();
    let mut trace = BufWriter::new(fs::File::create(output.with_extension("ndjson")).unwrap());
    let mut games = Vec::new();
    for seed in seeds {
        let cards = deck(seed);
        let ordered_deck: Vec<_> = cards.iter().enumerate().map(|(id, card)|
            json!({"cardId": id, "suit": card.suit.value(), "rank": card.rank.value()})).collect();
        write_line(
            &mut trace,
            &json!({"event": "initial", "seed": seed, "codeSha": sha,
            "shuffle": SHUFFLE, "orderedDeck": ordered_deck}),
        );
        let mut state = FullState::new_variant(3, variant(), cards.clone()).unwrap();
        let started = Instant::now();
        let mut actions = Vec::new();
        let mut labels = Vec::new();
        let mut error = None;
        let mut counts = [0_usize; 3];
        let mut total_move_seconds = 0.0_f64;
        let mut max_move_seconds = 0.0_f64;
        let mut decisions = 0;
        while !state.is_terminal() && state.turn() < 200 {
            write_line(
                &mut trace,
                &json!({"event": "beforeDecision", "seed": seed,
                "turn": state.turn(), "player": state.current_player().index(),
                "actionsApplied": actions.len()}),
            );
            let move_started = Instant::now();
            let result = catch_unwind(AssertUnwindSafe(|| {
                let view = state
                    .physical_view_for(state.current_player())
                    .ok_or_else(|| "missing acting view".to_owned())?;
                decide(view)
            }));
            let seconds = move_started.elapsed().as_secs_f64();
            total_move_seconds += seconds;
            max_move_seconds = max_move_seconds.max(seconds);
            decisions += 1;
            let action = match result {
                Ok(Ok(action)) => action,
                Ok(Err(cause)) => {
                    error = Some(cause);
                    break;
                }
                Err(payload) => {
                    let detail = payload
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| payload.downcast_ref::<&str>().copied())
                        .unwrap_or("non-string panic");
                    error = Some(format!("decision panic: {detail}"));
                    break;
                }
            };
            let label = action_json(action);
            write_line(
                &mut trace,
                &json!({"event": "selected", "seed": seed,
                "turn": state.turn(), "action": label, "moveSeconds": seconds}),
            );
            let applied = catch_unwind(AssertUnwindSafe(|| state.apply_physical(action)));
            match applied {
                Ok(Ok(_)) => {}
                Ok(Err(cause)) => {
                    error = Some(format!("apply: {cause}"));
                    break;
                }
                Err(_) => {
                    error = Some("apply panic".into());
                    break;
                }
            }
            counts[match action {
                PhysicalAction::Play(_) => 0,
                PhysicalAction::Discard(_) => 1,
                PhysicalAction::Clue { .. } => 2,
            }] += 1;
            actions.push(action);
            labels.push(label);
            write_line(
                &mut trace,
                &json!({"event": "applied", "seed": seed,
                "turn": state.turn(), "stackScore": state.score(), "strikes": state.strikes()}),
            );
        }
        if !state.is_terminal() && error.is_none() {
            error = Some("safety cap: 200 turns".into());
        }
        let reconstruction = catch_unwind(AssertUnwindSafe(|| {
            let mut replay =
                FullState::new_variant(3, variant(), cards).map_err(|e| e.to_string())?;
            for action in actions {
                replay.apply_physical(action).map_err(|e| e.to_string())?;
            }
            if replay != state {
                return Err("full simulator state mismatch".to_owned());
            }
            Ok::<_, String>(())
        }));
        let reconstruction_error = match reconstruction {
            Ok(Ok(())) => None,
            Ok(Err(cause)) => Some(cause),
            Err(_) => Some("reconstruction panic".into()),
        };
        let game = json!({"seed": seed, "variant": "Rainbow6Suits", "players": 3,
            "policy": "ConventionAgnosticPolicy (no clues; not H-Group)", "codeSha": sha,
            "shuffle": SHUFFLE, "orderedDeck": ordered_deck,
            "completed": state.is_terminal(), "terminalReason": format!("{:?}", state.status()),
            "officialFinalScore": state.final_score(), "stackScore": state.score(),
            "strikes": state.strikes(), "turns": state.turn(),
            "actionCounts": {"Play": counts[0], "Discard": counts[1], "Clue": counts[2]},
            "decisions": decisions, "totalMoveSeconds": total_move_seconds,
            "maxMoveSeconds": max_move_seconds, "elapsedSeconds": started.elapsed().as_secs_f64(),
            "observerRedactionCheckedEveryDecision": true,
            "reconstructionMatches": reconstruction_error.is_none(),
            "reconstructionError": reconstruction_error, "error": error, "actions": labels});
        write_line(&mut trace, &json!({"event": "result", "game": game}));
        games.push(game);
        fs::write(
            &output,
            serde_json::to_vec_pretty(&json!({"schemaVersion": 1, "games": games})).unwrap(),
        )
        .unwrap();
    }
    assert!(
        games.iter().all(|game| game["error"].is_null()
            && game["reconstructionError"].is_null()
            && game["completed"] == true),
        "pilot software failure; inspect persisted report (strikeout alone is not failure)"
    );
}
