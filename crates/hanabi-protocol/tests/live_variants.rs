//! Artificial scrubbed streams test physical codecs/bookkeeping, not strategy.
use hanabi_core::{
    Card, CardId, CardRank, Clue, ClueColorId, ClueRank, EndReason, GameStatus, GenericClue,
    ObservedEvent, PhysicalAction, PlayerId, PlayerView, SuitId, VariantId, VariantSpec,
};
use hanabi_protocol::{
    HanabiLiveActionCommand, HanabiLiveSessionState, HanabiLiveSnapshot, LiveSnapshotError,
};
use serde_json::{Value, json};

const VARIANTS: [(&str, VariantId, usize); 9] = [
    ("No Variant", VariantId::NoVariant, 50),
    ("Black (6 Suits)", VariantId::Black6Suits, 55),
    ("Rainbow (6 Suits)", VariantId::Rainbow6Suits, 60),
    ("White (6 Suits)", VariantId::White6Suits, 60),
    ("Pink (6 Suits)", VariantId::Pink6Suits, 60),
    ("Brown (6 Suits)", VariantId::Brown6Suits, 60),
    ("Prism (6 Suits)", VariantId::Prism6Suits, 60),
    ("Omni (6 Suits)", VariantId::Omni6Suits, 60),
    ("Null (6 Suits)", VariantId::Null6Suits, 60),
];

fn draw(player: usize, order: usize, suit: i16, rank: i16) -> Value {
    json!({"type": "draw", "playerIndex": player, "order": order,
        "suitIndex": suit, "rank": rank})
}

fn snapshot(name: &str) -> Value {
    let actions = (0..5)
        .map(|order| draw(0, order, -1, -1))
        .chain((5..10).map(|order| draw(1, order, 1, i16::try_from(order - 4).unwrap())))
        .collect::<Vec<_>>();
    json!({"tableID": 17, "playerNames": ["Bot", "Alice"], "ourPlayerIndex": 0,
        "options": {"variantName": name}, "actions": actions})
}

fn physical(snapshot: &Value) -> Result<PlayerView<GenericClue>, LiveSnapshotError> {
    HanabiLiveSnapshot::from_json(&snapshot.to_string())?.physical_player_view()
}

fn push(snapshot: &mut Value, action: Value) {
    snapshot["actions"].as_array_mut().unwrap().push(action);
}

fn clue(kind: u8, value: i16, touched: &[usize], target: usize) -> Value {
    json!({"type": "clue", "clue": {"type": kind, "value": value},
        "giver": 1 - target, "target": target, "list": touched, "turn": 0})
}

fn initialize(snapshot: &Value) -> String {
    json!({"kind": "initialize", "snapshot": snapshot}).to_string()
}

fn append(actions: &[Value]) -> String {
    json!({"kind": "append", "tableID": 17, "actions": actions}).to_string()
}

#[test]
fn all_exact_names_reconstruct_the_catalog_and_guard_legacy_policy() {
    for (name, id, total) in VARIANTS {
        let snapshot = snapshot(name);
        let view = physical(&snapshot).unwrap();
        assert_eq!(view.variant, VariantSpec::from_catalog(id), "{name}");
        assert_eq!(view.deck_size, total - 10, "{name}");
        assert_eq!(view.play_stacks.len(), if total == 50 { 5 } else { 6 });
        assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
        let legacy = HanabiLiveSnapshot::from_json(&snapshot.to_string())
            .unwrap()
            .player_view();
        if id == VariantId::NoVariant {
            assert_eq!(legacy.unwrap(), view.try_standard().unwrap());
        } else {
            assert!(
                matches!(legacy, Err(LiveSnapshotError::PolicyUnsupportedVariant(value)) if value == name)
            );
        }
    }
}

#[test]
fn unsupported_names_are_not_resolved_by_heuristics() {
    for name in [
        "Rainbow",
        "Rainbow (5 Suits)",
        "rainbow (6 Suits)",
        "No Variant ",
        "Unknown",
    ] {
        assert!(
            matches!(physical(&snapshot(name)), Err(LiveSnapshotError::UnsupportedVariant(value)) if value == name)
        );
    }
}

#[test]
fn sixth_suit_draws_and_public_reveals_use_physical_suits() {
    for (name, _, _) in &VARIANTS[1..] {
        let mut snapshot = snapshot(name);
        push(&mut snapshot, draw(1, 10, 5, 1));
        push(&mut snapshot, draw(0, 11, -1, -1));
        // Even accidentally unsanitized own draws must not disclose identities.
        push(&mut snapshot, draw(0, 12, 5, 2));
        let view = physical(&snapshot).unwrap();
        let identity = Card::new(SuitId::new(5), CardRank::new(1));
        assert_eq!(view.hands[1].last().unwrap().identity, Some(identity));
        assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
        assert!(
            matches!(view.history[0].event, ObservedEvent::Drew { identity: Some(card), .. } if card == identity)
        );
        for entry in &view.history[1..] {
            assert!(matches!(
                entry.event,
                ObservedEvent::Drew { identity: None, .. }
            ));
        }
        push(
            &mut snapshot,
            json!({"type": "play", "playerIndex": 1, "order": 10, "suitIndex": 5, "rank": 1}),
        );
        push(
            &mut snapshot,
            json!({"type": "discard", "playerIndex": 0, "order": 11, "suitIndex": 5, "rank": 3, "failed": false}),
        );
        let view = physical(&snapshot).unwrap();
        assert_eq!(view.play_stacks[5], vec![(CardId::new(10), identity)]);
        assert_eq!(
            view.discard_pile,
            vec![(CardId::new(11), Card::new(SuitId::new(5), CardRank::new(3)))]
        );
    }
}

#[test]
fn generic_clue_history_records_authoritative_touches_and_physical_facts() {
    // Each target has a sixth-suit 3 (order 5) and ordinary yellow 2..5.
    // Lists are public server evidence, never reconstructed own-hand truth.
    let cases: &[(&str, u8, i16, &[usize], bool)] = &[
        ("Black (6 Suits)", 0, 5, &[5], true),
        ("Rainbow (6 Suits)", 0, 0, &[5], true),
        ("White (6 Suits)", 0, 1, &[6, 7, 8, 9], false),
        ("White (6 Suits)", 1, 3, &[5, 7], true),
        ("Pink (6 Suits)", 1, 2, &[5, 6], true),
        ("Pink (6 Suits)", 0, 5, &[5], true),
        ("Brown (6 Suits)", 1, 3, &[7], false),
        ("Brown (6 Suits)", 0, 5, &[5], true),
        ("Prism (6 Suits)", 0, 2, &[5], true),
        ("Prism (6 Suits)", 0, 1, &[6, 7, 8, 9], false),
        ("Omni (6 Suits)", 0, 0, &[5], true),
        ("Omni (6 Suits)", 1, 2, &[5, 6], true),
        ("Null (6 Suits)", 0, 1, &[6, 7, 8, 9], false),
        ("Null (6 Suits)", 1, 3, &[7], false),
    ];
    for &(name, kind, label, orders, sixth_touched) in cases {
        let mut snapshot = snapshot(name);
        snapshot["actions"][5] = draw(1, 5, 5, 3);
        push(&mut snapshot, clue(kind, label, orders, 1));
        let view = physical(&snapshot).unwrap();
        let generic = if kind == 0 {
            GenericClue::Color(ClueColorId::new(u8::try_from(label).unwrap()))
        } else {
            GenericClue::Rank(ClueRank::new(u8::try_from(label).unwrap()))
        };
        let ObservedEvent::Clued {
            clue,
            touched,
            untouched,
            ..
        } = &view.history[0].event
        else {
            panic!("clue history missing")
        };
        assert_eq!(*clue, generic);
        assert_eq!(
            touched,
            &orders.iter().copied().map(CardId::new).collect::<Vec<_>>()
        );
        assert_eq!(touched.len() + untouched.len(), 5);
        let sixth = view.hands[1][0];
        assert_eq!(
            sixth.clues.has_positive_physical_clue(generic),
            sixth_touched
        );
        assert_eq!(
            sixth.clues.has_negative_physical_clue(generic),
            !sixth_touched
        );
        let identity = view
            .variant
            .identities()
            .id_for(SuitId::new(5), CardRank::new(3))
            .unwrap();
        assert_ne!(
            sixth.clues.identity_mask_in(&view.variant) & (1 << identity.value()),
            0,
            "{name}"
        );
        for card in &view.hands[1] {
            assert_eq!(
                card.clues.has_positive_physical_clue(generic),
                orders.contains(&card.id.index())
            );
        }
    }
}

#[test]
fn public_clues_never_fill_own_identities() {
    let mut snapshot = snapshot("Rainbow (6 Suits)");
    push(&mut snapshot, clue(0, 0, &[0, 2], 0));
    push(&mut snapshot, clue(1, 3, &[2], 0));
    let view = physical(&snapshot).unwrap();
    assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
    assert!(
        view.hands[0][2]
            .clues
            .has_positive_physical_clue(GenericClue::Color(ClueColorId::new(0)))
    );
    assert!(
        view.hands[0][2]
            .clues
            .has_positive_physical_clue(GenericClue::Rank(ClueRank::new(3)))
    );
    assert!(
        view.hands[0][1]
            .clues
            .has_negative_physical_clue(GenericClue::Color(ClueColorId::new(0)))
    );
    assert_eq!(view.history.len(), 2);
}

#[test]
fn invalid_identities_clues_and_orders_use_the_active_domains() {
    for (name, _, total) in VARIANTS {
        for (suit, rank) in [(6, 1), (-2, 1), (0, 0), (0, 6), (-1, 1), (5, -1)] {
            let mut snapshot = snapshot(name);
            push(&mut snapshot, draw(1, 10, suit, rank));
            assert!(physical(&snapshot).is_err(), "{name}: {suit}, {rank}");
        }
        for kind in ["draw", "play", "discard"] {
            let mut snapshot = snapshot(name);
            push(
                &mut snapshot,
                json!({"type": kind, "playerIndex": 0, "order": total, "suitIndex": 0, "rank": 1, "failed": false}),
            );
            assert!(
                matches!(physical(&snapshot), Err(LiveSnapshotError::InvalidCardOrder(value)) if value == total)
            );
        }
        for (kind, value) in [(0, -1), (0, 6), (1, 0), (1, 6), (0, 256), (1, 256)] {
            let mut snapshot = snapshot(name);
            push(&mut snapshot, clue(kind, value, &[5], 1));
            assert!(matches!(
                physical(&snapshot),
                Err(LiveSnapshotError::InvalidClueColor(_) | LiveSnapshotError::InvalidClueRank(_))
            ));
        }
    }
    for name in [
        "No Variant",
        "Rainbow (6 Suits)",
        "White (6 Suits)",
        "Prism (6 Suits)",
        "Omni (6 Suits)",
        "Null (6 Suits)",
    ] {
        let mut snapshot = snapshot(name);
        push(&mut snapshot, clue(0, 5, &[5], 1));
        assert!(matches!(
            physical(&snapshot),
            Err(LiveSnapshotError::InvalidClueColor(5))
        ));
    }
    let mut standard = snapshot("No Variant");
    push(&mut standard, draw(1, 10, 5, 1));
    assert!(matches!(
        physical(&standard),
        Err(LiveSnapshotError::InvalidSuit(5))
    ));
    let mut hidden_teammate = snapshot("Null (6 Suits)");
    push(&mut hidden_teammate, draw(1, 10, -1, -1));
    assert!(matches!(
        physical(&hidden_teammate),
        Err(LiveSnapshotError::HiddenTeammateCard { .. })
    ));
}

#[test]
fn deck_exhaustion_and_final_round_use_all_variant_cards() {
    for (name, _, total) in VARIANTS {
        let mut snapshot = snapshot(name);
        // Codec invariant: walk every unique order, replacing a hidden own card.
        for order in 10..total {
            let removed = if order < 15 { order - 10 } else { order - 5 };
            push(
                &mut snapshot,
                json!({"type": "discard", "playerIndex": 0, "order": removed, "suitIndex": 0, "rank": 1, "failed": false}),
            );
            push(&mut snapshot, draw(0, order, -1, -1));
        }
        let view = physical(&snapshot).unwrap();
        assert_eq!(view.deck_size, 0);
        assert_eq!(view.final_turns_remaining, Some(2));
        assert_eq!(view.hands[0].last().unwrap().id.index(), total - 1);
        push(
            &mut snapshot,
            json!({"type": "turn", "num": 2, "currentPlayerIndex": 1}),
        );
        assert_eq!(physical(&snapshot).unwrap().final_turns_remaining, Some(1));
    }
}

#[test]
fn variant_sessions_append_transactionally_and_keep_metadata_fixed() {
    let snapshot = snapshot("Black (6 Suits)");
    let mut session = HanabiLiveSessionState::new();
    let (table, initial) = session.apply_physical_json(&initialize(&snapshot)).unwrap();
    assert_eq!(table, 17);
    let actions = [draw(1, 50, 5, 1), clue(0, 5, &[50], 1)];
    let (_, updated) = session.apply_physical_json(&append(&actions)).unwrap();
    assert_eq!(updated.deck_size, 44);
    let mut complete = snapshot.clone();
    for action in actions {
        push(&mut complete, action);
    }
    assert_eq!(updated, physical(&complete).unwrap());
    for malformed in [
        append(&[draw(0, 51, -1, -1), draw(1, 55, 5, 1)]),
        append(&[clue(0, 5, &[50, 54], 1)]),
        append(&[
            draw(0, 51, -1, -1),
            json!({"type": "play", "playerIndex": 0, "order": 51, "suitIndex": 5, "rank": 3}),
        ]),
        append(&[json!({"type": "play", "playerIndex": 1, "order": 50, "suitIndex": 5, "rank": 2})]),
        json!({"kind": "append", "tableID": 17, "actions": [], "options": {"variantName": "No Variant"}}).to_string(),
        json!({"kind": "append", "tableID": 99, "actions": []}).to_string(),
    ] {
        assert!(session.apply_physical_json(&malformed).is_err());
        assert_eq!(session.apply_physical_json(&append(&[])).unwrap().1, updated);
    }
    let mut invalid_init = snapshot.clone();
    invalid_init["actions"].as_array_mut().unwrap().pop();
    assert!(
        session
            .apply_physical_json(&initialize(&invalid_init))
            .is_err()
    );
    assert_eq!(
        session.apply_physical_json(&append(&[])).unwrap().1,
        updated
    );
    assert_eq!(initial.variant, updated.variant);
    assert!(matches!(
        session.apply_json(&append(&[draw(0, 51, -1, -1)])),
        Err(LiveSnapshotError::PolicyUnsupportedVariant(_))
    ));
    assert_eq!(
        session.apply_physical_json(&append(&[])).unwrap().1,
        updated
    );
}

#[test]
fn legacy_initialize_rejects_variants_without_replacing_previous_session() {
    let mut session = HanabiLiveSessionState::new();
    let (_, standard) = session
        .apply_json(&initialize(&snapshot("No Variant")))
        .unwrap();
    for (name, _, _) in &VARIANTS[1..] {
        assert!(matches!(
            session.apply_json(&initialize(&snapshot(name))),
            Err(LiveSnapshotError::PolicyUnsupportedVariant(_))
        ));
        assert_eq!(session.apply_json(&append(&[])).unwrap().1, standard);
    }
    let mut snapshot = snapshot("No Variant");
    push(&mut snapshot, clue(0, 1, &[5, 6, 7, 8, 9], 1));
    push(&mut snapshot, clue(1, 2, &[6], 1));
    let legacy = HanabiLiveSnapshot::from_json(&snapshot.to_string())
        .unwrap()
        .player_view()
        .unwrap();
    assert_eq!(legacy, physical(&snapshot).unwrap().try_standard().unwrap());
    assert!(matches!(
        legacy.history[0].event,
        ObservedEvent::Clued {
            clue: Clue::Suit(_),
            ..
        }
    ));
    assert!(matches!(
        legacy.history[1].event,
        ObservedEvent::Clued {
            clue: Clue::Rank(_),
            ..
        }
    ));
}

#[test]
fn physical_commands_preserve_card_orders_and_clue_labels() {
    let target = PlayerId::new(1);
    for (action, expected) in [
        (
            PhysicalAction::Play(CardId::new(59)),
            json!({"tableID": 9, "type": 0, "target": 59}),
        ),
        (
            PhysicalAction::Discard(CardId::new(54)),
            json!({"tableID": 9, "type": 1, "target": 54}),
        ),
        (
            PhysicalAction::Clue {
                target,
                clue: GenericClue::Color(ClueColorId::new(5)),
            },
            json!({"tableID": 9, "type": 2, "target": 1, "value": 5}),
        ),
        (
            PhysicalAction::Clue {
                target,
                clue: GenericClue::Rank(ClueRank::new(3)),
            },
            json!({"tableID": 9, "type": 3, "target": 1, "value": 3}),
        ),
    ] {
        assert_eq!(
            serde_json::to_value(HanabiLiveActionCommand::from_physical_action(9, action)).unwrap(),
            expected
        );
    }
}

#[test]
fn perfect_score_and_completion_tokens_follow_the_variant() {
    for (name, id, _) in VARIANTS {
        let variant = VariantSpec::from_catalog(id);
        let mut snapshot = snapshot(name);
        // Initial own cards are red 1..5 (redacted); teammate has yellow 1..5.
        // Play one copy of every identity in stack order, drawing future suits
        // into our hand with exact own-hand redaction.
        let mut next_order = 10;
        for suit in 0..variant.stack_suits().len() {
            push(&mut snapshot, json!({"type": "status", "clues": 4}));
            for rank in 1..=5 {
                let (player, order) = match suit {
                    0 => (0, rank - 1),
                    1 => (1, rank + 4),
                    _ => {
                        let order = next_order;
                        next_order += 1;
                        push(&mut snapshot, draw(0, order, -1, -1));
                        (0, order)
                    }
                };
                push(
                    &mut snapshot,
                    json!({"type": "play", "playerIndex": player, "order": order, "suitIndex": suit, "rank": rank}),
                );
                assert_eq!(
                    physical(&snapshot).unwrap().clue_tokens,
                    if rank == 5 { 5 } else { 4 }
                );
            }
            if suit == 4 && id != VariantId::NoVariant {
                let mut stopped_at_25 = snapshot.clone();
                push(&mut stopped_at_25, json!({"type": "gameOver"}));
                assert_eq!(
                    physical(&stopped_at_25).unwrap().status,
                    GameStatus::Finished(EndReason::FinalRoundComplete)
                );
            }
        }
        push(&mut snapshot, json!({"type": "gameOver"}));
        let view = physical(&snapshot).unwrap();
        assert_eq!(
            view.play_stacks.iter().map(Vec::len).sum::<usize>(),
            variant.max_score()
        );
        assert_eq!(view.status, GameStatus::Finished(EndReason::PerfectScore));
    }
}
