//! Opt-in deterministic strength benchmark. See docs/self-play.md.
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use hanabi_core::{Action, Clue};
use hanabi_protocol::HanabiLiveReplay;
use hanabi_search::{
    H_GROUP_RULESET_REVISION, HGroupProfile, InformationSet, PlannerConfig, PlanningObjective,
    SupportedConvention, plan_move,
};
use serde::{Deserialize, Serialize};

const NAME: &str = "h_group_max_self_play_200";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameResult {
    seed: String,
    score: u8,
    stack_score: u8,
    strikes: u8,
    turns: u32,
    elapsed_seconds: f64,
    error: Option<String>,
    actions: Vec<serde_json::Value>,
    #[serde(default)]
    deadline_turns: Vec<u32>,
    #[serde(default)]
    failed_stage: Option<String>,
    #[serde(default)]
    failed_observer: Option<usize>,
    #[serde(default)]
    legal_replay_parity: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Baseline {
    schema_version: u8,
    name: String,
    convention: String,
    objective: String,
    exact_world_limit: u64,
    exact_node_limit: u64,
    max_turns: u32,
    ruleset_revision: String,
    total_score: u32,
    perfect_games: usize,
    /// Nonzero means this is a provisional measurement, not a clean baseline.
    engine_errors: usize,
    scores: Vec<(String, u8)>,
}

fn config() -> PlannerConfig {
    PlannerConfig {
        objective: PlanningObjective::PerfectScore,
        ..PlannerConfig::default()
    }
}

fn action_json(action: Action) -> serde_json::Value {
    match action {
        Action::Play(card) => serde_json::json!({"type": 0, "target": card.index()}),
        Action::Discard(card) => serde_json::json!({"type": 1, "target": card.index()}),
        Action::Clue { target, clue } => match clue {
            Clue::Suit(suit) => {
                serde_json::json!({"type": 2, "target": target.index(), "value": suit.index()})
            }
            Clue::Rank(rank) => {
                serde_json::json!({"type": 3, "target": target.index(), "value": rank.number()})
            }
        },
    }
}

fn play(seed: usize, active: &std::path::Path) -> GameResult {
    play_case(
        &format!("p4v0s{seed}"),
        HGroupProfile::Max,
        config(),
        active,
        None,
    )
}

fn play_case(
    seed: &str,
    profile: HGroupProfile,
    planner: PlannerConfig,
    active: &std::path::Path,
    deadline: Option<Instant>,
) -> GameResult {
    let started = Instant::now();
    let json = serde_json::json!({"seed": seed, "actions": []});
    let mut state = HanabiLiveReplay::from_json(&json.to_string())
        .unwrap()
        .state_at_turn(0)
        .unwrap();
    let mut actions = Vec::new();
    let mut error = None;
    let mut failed_stage = None;
    let mut failed_observer = None;
    let mut deadline_turns = Vec::new();
    // Only the simulator has the seed and FullState. Decisions receive the
    // acting player's permitted observation and deterministic search budgets.
    while !state.is_terminal() {
        // Cheap seed + action replay, written before planning so even a slow
        // or interrupted decision has an independently reproducible position.
        fs::write(
            active.join(format!("{seed}.json")),
            serde_json::json!({
                "seed": seed, "actions": actions,
                "profile": format!("{profile:?}"), "moveTimeMs": planner.move_time_limit.as_millis(),
                "nextTurn": state.turn() + 1, "observer": state.current_player().index(),
                "stage": "plan_move",
            })
            .to_string(),
        )
        .unwrap();
        failed_observer = Some(state.current_player().index());
        if deadline.is_some_and(|limit| Instant::now() >= limit) {
            failed_stage = Some("total_deadline".to_owned());
            error = Some("smoke total deadline reached".to_owned());
            break;
        }
        if state.turn() >= 200 {
            failed_stage = Some("turn_bound".to_owned());
            error = Some("game exceeded 200 turns without reaching a terminal state".to_owned());
            break;
        }
        let view = state.view_for(state.current_player()).unwrap();
        let decision = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let convention = SupportedConvention::HGroup(profile);
            let information = InformationSet::new(&view).map_err(|error| error.to_string())?;
            plan_move(&information, convention, planner).map_err(|error| error.to_string())
        }));
        match decision {
            Ok(Ok(result)) => {
                if result.budget_exhausted {
                    deadline_turns.push(state.turn() + 1);
                }
                if let Err(cause) = state.apply(result.best_action) {
                    failed_stage = Some("apply".to_owned());
                    error = Some(format!("illegal engine action: {cause}"));
                    break;
                }
                actions.push(action_json(result.best_action));
            }
            Ok(Err(cause)) => {
                failed_stage = Some("plan_move".to_owned());
                error = Some(cause);
                break;
            }
            Err(_) => {
                failed_stage = Some("plan_move_panic".to_owned());
                error = Some("engine panicked (see test stderr)".to_owned());
                break;
            }
        }
    }
    let replay = serde_json::json!({"seed": seed, "actions": actions});
    let legal_replay_parity = HanabiLiveReplay::from_json(&replay.to_string())
        .and_then(|replay| replay.replay())
        .is_ok_and(|replayed| replayed == state);
    if error.is_none() {
        failed_observer = None;
    }
    GameResult {
        seed: seed.to_owned(),
        score: state.final_score().unwrap_or(0),
        stack_score: state.score(),
        strikes: state.strikes(),
        turns: state.turn(),
        elapsed_seconds: started.elapsed().as_secs_f64(),
        error,
        actions,
        deadline_turns,
        failed_stage,
        failed_observer,
        legal_replay_parity,
    }
}

fn setting(name: &str, default: usize) -> usize {
    env::var(name).map_or(default, |value| {
        value.parse().expect("benchmark setting must be an integer")
    })
}

fn baseline(games: &[GameResult]) -> Baseline {
    Baseline {
        schema_version: 1,
        name: NAME.to_owned(),
        convention: "h-group:max".to_owned(),
        objective: "perfect-score".to_owned(),
        exact_world_limit: config().exact_world_limit,
        exact_node_limit: config().exact_node_limit,
        max_turns: 200,
        ruleset_revision: H_GROUP_RULESET_REVISION.to_owned(),
        total_score: games.iter().map(|game| u32::from(game.score)).sum(),
        perfect_games: games.iter().filter(|game| game.score == 25).count(),
        engine_errors: games.iter().filter(|game| game.error.is_some()).count(),
        scores: games
            .iter()
            .map(|game| (game.seed.clone(), game.score))
            .collect(),
    }
}

#[test]
#[ignore = "200 complete self-play games; run scripts/check-self-play.sh"]
fn h_group_max_self_play_200() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let start = setting("HANABI_SELF_PLAY_START", 1);
    let count = setting("HANABI_SELF_PLAY_GAMES", 200);
    let workers = setting("HANABI_SELF_PLAY_WORKERS", 4).min(count);
    assert!(count > 0 && workers > 0);
    let update = env::var_os("HANABI_SELF_PLAY_UPDATE").is_some();
    assert!(
        !update || (start == 1 && count == 200),
        "only the full suite can update the baseline"
    );
    let output = env::var_os("HANABI_SELF_PLAY_REPORT").map_or_else(
        || root.join("target/self-play/report.json"),
        |path| root.join(path),
    );
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let active = output.with_extension("active");
    fs::create_dir_all(&active).unwrap();
    let mut checkpoint = BufWriter::new(fs::File::create(output.with_extension("ndjson")).unwrap());
    let next = AtomicUsize::new(start);
    let started = Instant::now();
    let mut games = thread::scope(|scope| {
        let (tx, rx) = mpsc::channel();
        for _ in 0..workers {
            let tx = tx.clone();
            let next = &next;
            let active = &active;
            scope.spawn(move || {
                loop {
                    let seed = next.fetch_add(1, Ordering::Relaxed);
                    if seed >= start + count {
                        break;
                    }
                    if tx.send(play(seed, active)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(tx);
        let mut results = Vec::with_capacity(count);
        for result in rx {
            serde_json::to_writer(&mut checkpoint, &result).unwrap();
            writeln!(checkpoint).unwrap();
            checkpoint.flush().unwrap();
            eprintln!(
                "[{}/{}] {}: score {} ({} turns, {:.2}s){}",
                results.len() + 1,
                count,
                result.seed,
                result.score,
                result.turns,
                result.elapsed_seconds,
                result
                    .error
                    .as_ref()
                    .map_or(String::new(), |error| format!(" ERROR: {error}"))
            );
            results.push(result);
        }
        results
    });
    games.sort_by_key(|game| {
        game.seed
            .trim_start_matches("p4v0s")
            .parse::<usize>()
            .unwrap()
    });
    finish_report(
        &root,
        &output,
        &games,
        count,
        workers,
        started.elapsed().as_secs_f64(),
        update,
        start == 1 && count == 200,
    );
}

#[allow(clippy::too_many_arguments)]
fn finish_report(
    root: &std::path::Path,
    output: &std::path::Path,
    games: &[GameResult],
    count: usize,
    workers: usize,
    seconds: f64,
    update: bool,
    full_suite: bool,
) {
    let measured = baseline(games);
    let errors = games.iter().filter(|game| game.error.is_some()).count();
    let strikeouts = games.iter().filter(|game| game.strikes == 3).count();
    let baseline_path =
        root.join("crates/hanabi-search/tests/fixtures/h_group_max_self_play_200.json");
    let previous = if baseline_path.exists() {
        Some(
            serde_json::from_str::<Baseline>(&fs::read_to_string(&baseline_path).unwrap()).unwrap(),
        )
    } else {
        None
    };
    let differences = previous.as_ref().map_or_else(Vec::new, |previous| {
        games
            .iter()
            .filter_map(|game| {
                previous
                    .scores
                    .iter()
                    .find(|(seed, _)| seed == &game.seed)
                    .filter(|(_, score)| *score != game.score)
                    .map(|(_, score)| {
                        serde_json::json!({"seed": game.seed, "before": score, "after": game.score,
                    "delta": i16::from(game.score) - i16::from(*score)})
                    })
            })
            .collect::<Vec<_>>()
    });
    let report = serde_json::json!({
        "summary": measured, "elapsedSeconds": seconds, "workers": workers,
        "engineErrors": errors, "strikeouts": strikeouts,
        "differences": differences, "games": games,
    });
    fs::write(
        output,
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .unwrap();
    eprintln!(
        "{}: total {}, perfect {}/{}, strikeouts {}, engine errors {}, elapsed {:.2}s; report {}",
        NAME,
        measured.total_score,
        measured.perfect_games,
        count,
        strikeouts,
        errors,
        seconds,
        output.display()
    );
    assert_eq!(games.len(), count, "every requested seed must finish");
    assert_eq!(
        errors, 0,
        "engine errors invalidate a clean strength baseline; inspect the report (aborted games receive zero credit)"
    );
    if update {
        fs::create_dir_all(baseline_path.parent().unwrap()).unwrap();
        fs::write(
            &baseline_path,
            serde_json::to_string_pretty(&measured).unwrap() + "\n",
        )
        .unwrap();
    } else if full_suite {
        let previous =
            previous.expect("no baseline; explicitly run with HANABI_SELF_PLAY_UPDATE=1");
        assert_no_regression(&previous, &measured);
    }
}

fn assert_no_regression(previous: &Baseline, measured: &Baseline) {
    assert_eq!(previous.schema_version, measured.schema_version);
    assert_eq!(previous.convention, measured.convention);
    assert_eq!(previous.objective, measured.objective);
    assert_eq!(previous.exact_world_limit, measured.exact_world_limit);
    assert_eq!(previous.exact_node_limit, measured.exact_node_limit);
    assert_eq!(previous.max_turns, measured.max_turns);
    assert_eq!(
        previous
            .scores
            .iter()
            .map(|(seed, _)| seed)
            .collect::<Vec<_>>(),
        measured
            .scores
            .iter()
            .map(|(seed, _)| seed)
            .collect::<Vec<_>>()
    );
    assert!(
        measured.total_score >= previous.total_score,
        "total score decreased: {} -> {}; inspect per-seed differences",
        previous.total_score,
        measured.total_score
    );
    assert!(
        measured.perfect_games >= previous.perfect_games,
        "perfect games decreased: {} -> {}",
        previous.perfect_games,
        measured.perfect_games
    );
}

#[test]
fn benchmark_rejects_score_and_perfect_game_decreases() {
    let mut previous = baseline(&[]);
    previous.total_score = 100;
    previous.perfect_games = 3;
    let mut measured = baseline(&[]);
    measured.total_score = 101;
    measured.perfect_games = 3;
    assert_no_regression(&previous, &measured);
    measured.total_score = 99;
    assert!(std::panic::catch_unwind(|| assert_no_regression(&previous, &measured)).is_err());
    measured.total_score = 101;
    measured.perfect_games = 2;
    assert!(std::panic::catch_unwind(|| assert_no_regression(&previous, &measured)).is_err());
    measured.perfect_games = 3;
    measured.exact_node_limit = 1;
    assert!(std::panic::catch_unwind(|| assert_no_regression(&previous, &measured)).is_err());
}

const MVP_CASES: [(&str, HGroupProfile); 3] = [
    (
        "p3v0s1",
        HGroupProfile::Level(hanabi_search::HGroupLevel::Level1),
    ),
    (
        "p4v0s1",
        HGroupProfile::Level(hanabi_search::HGroupLevel::Level4),
    ),
    ("p4v0s2", HGroupProfile::Max),
];
const MVP_MOVE_TIME: Duration = Duration::from_secs(2);
const MVP_TOTAL_TIME: Duration = Duration::from_secs(180);

#[test]
#[ignore = "bounded No Variant diagnostic; run scripts/check-no-variant-smoke.py"]
fn no_variant_mvp_smoke() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/no-variant-smoke");
    fs::create_dir_all(&output).unwrap();
    let mut report = fs::File::create(output.join("games.ndjson")).unwrap();
    let deadline = Instant::now() + MVP_TOTAL_TIME;
    for (seed, profile) in MVP_CASES {
        let planner = PlannerConfig {
            move_time_limit: MVP_MOVE_TIME,
            ..config()
        };
        let result = play_case(seed, profile, planner, &output, Some(deadline));
        serde_json::to_writer(
            &mut report,
            &serde_json::json!({
                "profile": format!("{profile:?}"), "moveTimeMs": MVP_MOVE_TIME.as_millis(),
                "totalTimeSeconds": MVP_TOTAL_TIME.as_secs(), "game": result,
            }),
        )
        .unwrap();
        writeln!(report).unwrap();
        report.flush().unwrap();
        assert!(result.legal_replay_parity, "illegal prefix: {seed}");
        assert!(
            result.error.is_none(),
            "first error: {seed}, turn {}, observer {:?}, stage {:?}: {:?}",
            result.turns + 1,
            result.failed_observer,
            result.failed_stage,
            result.error
        );
    }
}

#[test]
fn mvp_smoke_configuration_and_action_codec() {
    assert_eq!(MVP_CASES.len(), 3);
    for (seed, _) in MVP_CASES {
        let replay = HanabiLiveReplay::from_json(
            &serde_json::json!({"seed": seed, "actions": []}).to_string(),
        )
        .unwrap();
        let mut state = replay.state_at_turn(0).unwrap();
        assert!((3..=4).contains(&state.hands().len()));
        // Choose a legal clue from visible faces, independent of strategy.
        let face = state.view_for(state.current_player()).unwrap().hands[1][0]
            .identity
            .unwrap();
        let action = Action::Clue {
            target: hanabi_core::PlayerId::new(1),
            clue: Clue::Rank(face.rank),
        };
        state.apply(action).unwrap();
        let encoded = serde_json::json!({"seed": seed, "actions": [action_json(action)]});
        assert_eq!(
            HanabiLiveReplay::from_json(&encoded.to_string())
                .unwrap()
                .replay()
                .unwrap(),
            state
        );
    }
}


/// Read-only validation branch only: a real complete game with the normal
/// production planner configuration. The hard CI time limit is external.
#[test]
#[ignore = "explicit full-game offline validation on the verification branch"]
fn offline_full_game_level4_p4v0s1() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/offline-full-games");
    fs::create_dir_all(&output).unwrap();
    let result = play_case(
        "p4v0s1",
        HGroupProfile::Level(hanabi_search::HGroupLevel::Level4),
        config(),
        &output,
        None,
    );
    fs::write(
        output.join("level4-result.json"),
        serde_json::to_string_pretty(&result).unwrap() + "\n",
    )
    .unwrap();
    eprintln!(
        "LEVEL4 offline: turns={} stack_score={} official_score={} strikes={} error={:?} elapsed={:.2}s",
        result.turns, result.stack_score, result.score,
        result.strikes, result.error, result.elapsed_seconds,
    );
    assert!(result.legal_replay_parity, "replayed game diverged");
    assert!(result.error.is_none(), "game failed: {:?}", result.error);
    assert!(result.turns > 0, "no gameplay");
}

#[test]
#[ignore = "explicit full-game offline validation on the verification branch"]
fn offline_full_game_max_p4v0s2() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("target/offline-full-games");
    fs::create_dir_all(&output).unwrap();
    let result = play_case(
        "p4v0s2",
        HGroupProfile::Max,
        config(),
        &output,
        None,
    );
    fs::write(
        output.join("max-result.json"),
        serde_json::to_string_pretty(&result).unwrap() + "\n",
    )
    .unwrap();
    eprintln!(
        "MAX offline: turns={} stack_score={} official_score={} strikes={} error={:?} elapsed={:.2}s",
        result.turns, result.stack_score, result.score,
        result.strikes, result.error, result.elapsed_seconds,
    );
    assert!(result.legal_replay_parity, "replayed game diverged");
    assert!(result.error.is_none(), "game failed: {:?}", result.error);
    assert!(result.turns > 0, "no gameplay");
}
