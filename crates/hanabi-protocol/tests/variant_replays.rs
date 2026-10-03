//! Offline codec and physical-rule tests; these deals make no strategy claims.
use hanabi_core::{
    Card, CardId, CardRank, ClueColorId, ClueRank, FullState, GameEvent, GenericClue,
    PhysicalAction, PlayerId, RuleError, SuitId, VariantId, VariantSpec,
};
use hanabi_protocol::{HanabiLiveReplay, ReplayError};
use serde_json::{Value, json};

const VARIANTS: [(&str, VariantId); 9] = [
    ("No Variant", VariantId::NoVariant),
    ("Black (6 Suits)", VariantId::Black6Suits),
    ("Rainbow (6 Suits)", VariantId::Rainbow6Suits),
    ("White (6 Suits)", VariantId::White6Suits),
    ("Pink (6 Suits)", VariantId::Pink6Suits),
    ("Brown (6 Suits)", VariantId::Brown6Suits),
    ("Prism (6 Suits)", VariantId::Prism6Suits),
    ("Omni (6 Suits)", VariantId::Omni6Suits),
    ("Null (6 Suits)", VariantId::Null6Suits),
];

// Independent wire decks: standard 3/2/2/2/1 copies, Black extras one each.
// Full decks are required by setup validation; no large JSON fixture is stored.
fn replay_json(name: &str, prefix: &[(u8, u8)], actions: Value) -> Value {
    let suits = if name == "No Variant" { 5 } else { 6 };
    let mut cards = Vec::new();
    for suit in 0..suits {
        for rank in 1..=5 {
            let copies = if suit == 5 && name == "Black (6 Suits)" {
                1
            } else {
                [3, 2, 2, 2, 1][usize::from(rank - 1)]
            };
            cards.extend(core::iter::repeat_n((suit, rank), copies));
        }
    }
    for (index, wanted) in prefix.iter().enumerate() {
        let found = index
            + cards[index..]
                .iter()
                .position(|card| card == wanted)
                .unwrap();
        cards.swap(index, found);
    }
    let deck = cards
        .iter()
        .map(|(suit, rank)| json!({"suitIndex": suit, "rank": rank}))
        .collect::<Vec<_>>();
    let mut replay = json!({"players": ["A", "B"], "deck": deck, "options": {"variant": name}});
    replay["actions"] = actions;
    replay
}

fn parse(json: &Value) -> HanabiLiveReplay {
    HanabiLiveReplay::from_json(&json.to_string()).unwrap()
}

fn color(value: u8) -> GenericClue {
    GenericClue::Color(ClueColorId::new(value))
}

fn rank(value: u8) -> GenericClue {
    GenericClue::Rank(ClueRank::new(value))
}

#[test]
fn exact_names_replay_explicit_decks_and_sixth_suit_card_orders() {
    for (name, id) in VARIANTS {
        let extra = if id == VariantId::NoVariant { 4 } else { 5 };
        let json = replay_json(
            name,
            &[(extra, 1), (3, 3), (0, 3), (2, 4), (4, 5), (0, 1)],
            json!([
                {"type": 0, "target": 0},
                {"type": 3, "target": 0, "value": 3},
                {"type": 1, "target": 1},
                {"type": 0, "target": 5}
            ]),
        );
        let replay = parse(&json);
        let initial = replay.state_at_turn(0).unwrap();
        assert_eq!(initial.variant().catalog_id(), Some(id), "{name}");
        assert_eq!(
            initial.card(CardId::new(0)),
            Some(Card::new(SuitId::new(extra), CardRank::new(1)))
        );
        let state = replay.replay().unwrap();
        assert_eq!(state.turn(), 4);
        assert_eq!(state.score(), 2);
        assert_eq!(state.play_stacks()[usize::from(extra)].len(), 1);
        assert_eq!(state.discard_pile(), &[CardId::new(1)]);
        assert_eq!(state, replay.replay().unwrap());
        assert_eq!(state, replay.state_at_turn(4).unwrap());
        state.validate().unwrap();
    }
}

#[test]
fn unsupported_names_are_typed_without_name_heuristics() {
    for name in [
        "Rainbow",
        "rainbow (6 Suits)",
        "Rainbow (5 Suits)",
        "Rainbow (6 Suits) ",
        "no-variant",
        "Unknown",
    ] {
        for deck in [Some(json!([])), None] {
            let mut json = json!({"players": ["A", "B"], "seed": "p2v0s1", "actions": [], "options": {"variant": name}});
            if let Some(deck) = deck {
                json["deck"] = deck;
            }
            assert!(
                matches!(HanabiLiveReplay::from_json(&json.to_string()), Err(ReplayError::UnsupportedVariant(value)) if value == name)
            );
        }
        // Direct Deserialize callers still receive typed semantic errors on replay.
        let json =
            json!({"players": ["A", "B"], "deck": [], "actions": [], "options": {"variant": name}});
        let replay: HanabiLiveReplay = serde_json::from_value(json).unwrap();
        assert!(
            matches!(replay.replay(), Err(ReplayError::UnsupportedVariant(value)) if value == name)
        );
    }
}

#[test]
fn seed_generation_remains_standard_only_and_explicit_decks_win() {
    for (name, _) in &VARIANTS[1..] {
        let json = json!({"players": ["A", "B"], "seed": "p2v0s1", "actions": [], "options": {"variant": name}});
        let error = HanabiLiveReplay::from_json(&json.to_string()).unwrap_err();
        assert!(matches!(error, ReplayError::Json(_)));
        assert!(
            error
                .to_string()
                .contains("seed generation only supports No Variant")
        );
        let mut explicit = replay_json(name, &[], json!([]));
        explicit["seed"] = json!("untrusted metadata");
        parse(&explicit).replay().unwrap();
    }
}

#[test]
fn black_deck_validation_uses_one_copy_of_each_extra_rank() {
    let valid = replay_json("Black (6 Suits)", &[], json!([]));
    assert_eq!(parse(&valid).deck.len(), 55);
    parse(&valid).replay().unwrap();
    let mut missing = valid.clone();
    missing["deck"].as_array_mut().unwrap().pop();
    assert!(matches!(
        parse(&missing).replay(),
        Err(ReplayError::Setup(_))
    ));
    let mut duplicate = valid.clone();
    duplicate["deck"][54] = duplicate["deck"][50].clone();
    assert!(matches!(
        parse(&duplicate).replay(),
        Err(ReplayError::Setup(_))
    ));
    let ordinary_copies = replay_json("Rainbow (6 Suits)", &[], json!([]));
    let mut too_many = ordinary_copies;
    too_many["options"]["variant"] = json!("Black (6 Suits)");
    assert!(matches!(
        parse(&too_many).replay(),
        Err(ReplayError::Setup(_))
    ));
}

fn clue_replay(name: &str, clue_type: u8, value: u8) -> HanabiLiveReplay {
    // Target IDs 5 and 8 are special 1 and 4; 6 is red 2, 7 blue 3, 9 yellow 4.
    let prefix = [
        (0, 1),
        (0, 1),
        (0, 1),
        (0, 3),
        (0, 4),
        (5, 1),
        (0, 2),
        (3, 3),
        (5, 4),
        (1, 4),
    ];
    parse(&replay_json(
        name,
        &prefix,
        json!([{"type": clue_type, "target": 1, "value": value}]),
    ))
}

fn assert_touched(name: &str, clue_type: u8, value: u8, expected: &[usize]) {
    let state = clue_replay(name, clue_type, value).replay().unwrap();
    let expected_clue = if clue_type == 2 {
        color(value)
    } else {
        rank(value)
    };
    let GameEvent::Clued {
        clue,
        touched,
        untouched,
        ..
    } = &state.history()[0].event
    else {
        panic!("clue event")
    };
    assert_eq!(*clue, expected_clue);
    assert_eq!(
        *touched,
        expected
            .iter()
            .copied()
            .map(CardId::new)
            .collect::<Vec<_>>(),
        "{name}: {expected_clue:?}"
    );
    assert_eq!(touched.len() + untouched.len(), 5);
    let view = state.physical_view_for(PlayerId::new(1)).unwrap();
    for card in &view.hands[1] {
        assert_eq!(
            card.clues.has_positive_physical_clue(expected_clue),
            touched.contains(&card.id)
        );
        assert_eq!(
            card.clues.has_negative_physical_clue(expected_clue),
            untouched.contains(&card.id)
        );
    }
    state.validate().unwrap();
}

#[test]
fn variant_color_clues_use_physical_touching_and_labels() {
    for (name, label, touched) in [
        ("Black (6 Suits)", 5, vec![5, 8]),
        ("Rainbow (6 Suits)", 0, vec![5, 6, 8]),
        ("White (6 Suits)", 0, vec![6]),
        ("Pink (6 Suits)", 5, vec![5, 8]),
        ("Brown (6 Suits)", 5, vec![5, 8]),
        ("Prism (6 Suits)", 0, vec![5, 6]),
        ("Prism (6 Suits)", 3, vec![7, 8]),
        ("Omni (6 Suits)", 0, vec![5, 6, 8]),
        ("Null (6 Suits)", 0, vec![6]),
    ] {
        assert_touched(name, 2, label, &touched);
    }
}

#[test]
fn variant_rank_clues_cover_pink_brown_omni_and_null() {
    for (name, label, touched) in [
        ("Black (6 Suits)", 1, vec![5]),
        ("Rainbow (6 Suits)", 1, vec![5]),
        ("White (6 Suits)", 1, vec![5]),
        ("Pink (6 Suits)", 2, vec![5, 6, 8]),
        ("Brown (6 Suits)", 2, vec![6]),
        ("Prism (6 Suits)", 1, vec![5]),
        ("Omni (6 Suits)", 2, vec![5, 6, 8]),
        ("Null (6 Suits)", 2, vec![6]),
    ] {
        assert_touched(name, 3, label, &touched);
    }
    for name in ["Brown (6 Suits)", "Null (6 Suits)"] {
        assert!(matches!(
            clue_replay(name, 3, 1).replay(),
            Err(ReplayError::IllegalAction {
                turn: 0,
                source: RuleError::ClueTouchesNoCards
            })
        ));
    }
}

#[test]
fn invalid_suit_indices_and_card_ranks_are_typed() {
    for (name, _) in VARIANTS {
        for value in [if name == "No Variant" { 5 } else { 6 }, 255] {
            let mut json = replay_json(name, &[], json!([]));
            json["deck"][0]["suitIndex"] = json!(value);
            assert!(
                matches!(parse(&json).replay(), Err(ReplayError::InvalidSuit(suit)) if suit == value)
            );
        }
        for value in [0, 6, 255] {
            let mut json = replay_json(name, &[], json!([]));
            json["deck"][0]["rank"] = json!(value);
            assert!(
                matches!(parse(&json).replay(), Err(ReplayError::InvalidRank(rank)) if rank == value)
            );
        }
    }
}

#[test]
fn clue_labels_are_validated_separately_from_card_domains() {
    for (name, id) in VARIANTS {
        let colors: &[u8] = if matches!(
            id,
            VariantId::Black6Suits | VariantId::Pink6Suits | VariantId::Brown6Suits
        ) {
            &[6, 255]
        } else {
            &[5, 6, 255]
        };
        for (kind, values) in [(2, colors), (3, &[0, 6, 255][..])] {
            for value in values {
                let json = replay_json(
                    name,
                    &[],
                    json!([{"type": kind, "target": 1, "value": value}]),
                );
                let expected = if kind == 2 {
                    color(*value)
                } else {
                    rank(*value)
                };
                assert!(
                    matches!(parse(&json).replay(), Err(ReplayError::InvalidClue(clue)) if clue == expected)
                );
            }
        }
    }
}

#[test]
fn nonstandard_state_at_turn_preserves_prefix_semantics() {
    let mut replay = clue_replay("Rainbow (6 Suits)", 2, 0);
    let prefix = replay.state_at_turn(1).unwrap();
    assert_eq!(prefix, replay.replay().unwrap());
    replay
        .actions
        .push(hanabi_protocol::hanabi_live::HanabiLiveAction {
            action_type: hanabi_protocol::HanabiLiveActionType::RankClue,
            target: 0,
            value: 0,
        });
    assert_eq!(prefix, replay.state_at_turn(1).unwrap());
    assert_eq!(replay.state_at_turn(0).unwrap().turn(), 0);
    assert!(matches!(replay.replay(), Err(ReplayError::InvalidClue(_))));
    replay.actions.pop();
    assert!(matches!(
        replay.state_at_turn(2),
        Err(ReplayError::TurnOutOfRange {
            requested: 2,
            available: 1
        })
    ));
}

#[test]
fn no_variant_defaults_match_legacy_state_and_actions() {
    let json = replay_json(
        "No Variant",
        &[(0, 1), (3, 3)],
        json!([
            {"type": 0, "target": 0}, {"type": 3, "target": 0, "value": 3}, {"type": 1, "target": 1}
        ]),
    );
    let replay = parse(&json);
    let deck = replay
        .deck
        .iter()
        .map(|card| Card::new(SuitId::new(card.suit_index), CardRank::new(card.rank)))
        .collect();
    let mut legacy = FullState::new_standard(2, deck).unwrap();
    for action in [
        PhysicalAction::Play(CardId::new(0)),
        PhysicalAction::Clue {
            target: PlayerId::new(0),
            clue: rank(3),
        },
        PhysicalAction::Discard(CardId::new(1)),
    ] {
        legacy
            .apply(hanabi_core::Action::try_from(action).unwrap())
            .unwrap();
    }
    assert_eq!(replay.replay().unwrap(), legacy);
    assert_eq!(legacy.variant(), &VariantSpec::no_variant());
    assert!(legacy.standard_history().is_ok());
    for options in [
        None,
        Some(json!({})),
        Some(json!({"variant": "No Variant"})),
    ] {
        let mut defaulted = json.clone();
        if let Some(options) = options {
            defaulted["options"] = options;
        } else {
            defaulted.as_object_mut().unwrap().remove("options");
        }
        assert_eq!(parse(&defaulted).replay().unwrap(), legacy);
    }
}
