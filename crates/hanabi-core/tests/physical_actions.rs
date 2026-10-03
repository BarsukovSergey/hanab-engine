//! Physical rules and codec invariants; these positions make no strategy claims.
use hanabi_core::{
    Action, Card, CardId, CardRank, Clue, ClueColorId, ClueRank, FullState, GameEvent, GenericClue,
    IdentitySpec, ObservedEvent, PhysicalAction, PlayerId, Rank, RuleError, Suit, SuitId,
    VariantId, VariantSpec, WorldTemplate, standard_deck,
};

fn color(value: u8) -> GenericClue {
    GenericClue::Color(ClueColorId::new(value))
}
fn rank(value: u8) -> GenericClue {
    GenericClue::Rank(ClueRank::new(value))
}
fn clue(target: u8, clue: GenericClue) -> PhysicalAction {
    PhysicalAction::Clue {
        target: PlayerId::new(target),
        clue,
    }
}
fn deck_for(spec: &VariantSpec, prefix: &[Card]) -> Vec<Card> {
    let mut deck = spec
        .identities()
        .identities()
        .iter()
        .flat_map(|identity| {
            core::iter::repeat_n(
                Card::new(identity.suit, identity.rank),
                usize::from(identity.copies),
            )
        })
        .collect::<Vec<_>>();
    for (index, wanted) in prefix.iter().enumerate() {
        let found = index
            + deck[index..]
                .iter()
                .position(|card| card == wanted)
                .unwrap();
        deck.swap(index, found);
    }
    deck
}

#[test]
fn standard_actions_and_clues_convert_losslessly() {
    for standard in Suit::ALL
        .map(Clue::Suit)
        .into_iter()
        .chain(Rank::ALL.map(Clue::Rank))
    {
        assert_eq!(Clue::try_from(GenericClue::from(standard)), Ok(standard));
        let action = Action::Clue {
            target: PlayerId::new(1),
            clue: standard,
        };
        assert_eq!(Action::try_from(PhysicalAction::from(action)), Ok(action));
    }
    for action in [
        Action::Play(CardId::new(2)),
        Action::Discard(CardId::new(3)),
    ] {
        assert_eq!(Action::try_from(PhysicalAction::from(action)), Ok(action));
    }
    for generic in [color(5), color(255), rank(0), rank(6), rank(255)] {
        assert!(Clue::try_from(generic).is_err());
        assert!(Action::try_from(clue(1, generic)).is_err());
    }
}

#[test]
fn no_variant_legacy_and_physical_apply_match() {
    let initial = FullState::new_standard(3, standard_deck()).unwrap();
    // Every initial clue/play, and every subsequent discard, uses one shared
    // physical transition implementation while preserving the legacy result.
    for action in initial.legal_actions() {
        let mut legacy = initial.clone();
        let mut physical = initial.clone();
        let left = legacy.apply(action).unwrap();
        let right = physical.apply_physical(action.into()).unwrap();
        assert_eq!(left.actor, right.actor);
        assert_eq!(left.action, Action::try_from(right.action).unwrap());
        assert_eq!(left.drawn, right.drawn);
        assert_eq!(left.status, right.status);
        assert_eq!(legacy, physical);
        assert_eq!(
            legacy.view_for(legacy.current_player()).unwrap(),
            physical
                .physical_view_for(physical.current_player())
                .unwrap()
                .try_standard()
                .unwrap()
        );
        for discard in legacy
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::Discard(_)))
        {
            let mut a = legacy.clone();
            let mut b = physical.clone();
            a.apply(discard).unwrap();
            b.apply_physical(discard.into()).unwrap();
            assert_eq!(a, b);
        }
    }
}

fn expected_order(state: &FullState, expected: &[Action]) {
    assert_eq!(state.legal_actions(), expected);
    let mut reused = vec![Action::Play(CardId::new(999))];
    state.legal_actions_into(&mut reused);
    assert_eq!(reused, expected);
    let physical = expected
        .iter()
        .copied()
        .map(PhysicalAction::from)
        .collect::<Vec<_>>();
    assert_eq!(state.legal_physical_actions(), physical);
    let view = state.view_for(state.current_player()).unwrap();
    assert_eq!(view.legal_actions(), expected);
    assert_eq!(view.legal_physical_actions(), physical);
    assert_eq!(
        state
            .physical_view_for(state.current_player())
            .unwrap()
            .legal_physical_actions(),
        physical
    );
}

#[test]
fn exact_no_variant_action_order_before_and_after_spending_clue() {
    let spec = VariantSpec::no_variant();
    let prefix = [
        Card::new(Suit::Red, Rank::One),
        Card::new(Suit::Yellow, Rank::One),
        Card::new(Suit::Green, Rank::One),
        Card::new(Suit::Blue, Rank::One),
        Card::new(Suit::Purple, Rank::One),
        Card::new(Suit::Red, Rank::One),
        Card::new(Suit::Purple, Rank::Five),
        Card::new(Suit::Blue, Rank::Three),
        Card::new(Suit::Yellow, Rank::Two),
        Card::new(Suit::Red, Rank::Four),
        Card::new(Suit::Green, Rank::Two),
        Card::new(Suit::Green, Rank::One),
        Card::new(Suit::Purple, Rank::Two),
        Card::new(Suit::Blue, Rank::One),
        Card::new(Suit::Yellow, Rank::Five),
    ];
    let mut state = FullState::new_standard(3, deck_for(&spec, &prefix)).unwrap();
    let mut initial = (0..5)
        .map(|id| Action::Play(CardId::new(id)))
        .collect::<Vec<_>>();
    initial.extend(
        [Suit::Red, Suit::Yellow, Suit::Blue, Suit::Purple].map(|suit| Action::Clue {
            target: PlayerId::new(1),
            clue: Clue::Suit(suit),
        }),
    );
    initial.extend(Rank::ALL.map(|rank| Action::Clue {
        target: PlayerId::new(1),
        clue: Clue::Rank(rank),
    }));
    initial.extend(
        [Suit::Yellow, Suit::Green, Suit::Blue, Suit::Purple].map(|suit| Action::Clue {
            target: PlayerId::new(2),
            clue: Clue::Suit(suit),
        }),
    );
    initial.extend([Rank::One, Rank::Two, Rank::Five].map(|rank| Action::Clue {
        target: PlayerId::new(2),
        clue: Clue::Rank(rank),
    }));
    expected_order(&state, &initial);
    assert_eq!(
        state
            .physical_view_for(PlayerId::new(1))
            .unwrap()
            .legal_physical_actions(),
        Vec::<PhysicalAction>::new()
    );

    state.apply_physical(clue(1, rank(2))).unwrap();
    let mut after = (5..10)
        .map(|id| Action::Play(CardId::new(id)))
        .collect::<Vec<_>>();
    after.extend((5..10).map(|id| Action::Discard(CardId::new(id))));
    after.extend(Suit::ALL.map(|suit| Action::Clue {
        target: PlayerId::new(0),
        clue: Clue::Suit(suit),
    }));
    after.push(Action::Clue {
        target: PlayerId::new(0),
        clue: Clue::Rank(Rank::One),
    });
    after.extend(
        [Suit::Yellow, Suit::Green, Suit::Blue, Suit::Purple].map(|suit| Action::Clue {
            target: PlayerId::new(2),
            clue: Clue::Suit(suit),
        }),
    );
    after.extend([Rank::One, Rank::Two, Rank::Five].map(|rank| Action::Clue {
        target: PlayerId::new(2),
        clue: Clue::Rank(rank),
    }));
    expected_order(&state, &after);
}

fn custom_spec() -> VariantSpec {
    VariantSpec::new(
        "Independent labels",
        vec![SuitId::new(7), SuitId::new(9)],
        vec![CardRank::new(2), CardRank::new(4)],
        vec![ClueColorId::new(200), ClueColorId::new(201)],
        vec![ClueRank::new(0), ClueRank::new(255)],
        vec![
            IdentitySpec {
                suit: SuitId::new(7),
                rank: CardRank::new(2),
                copies: 3,
                clue_colors: vec![ClueColorId::new(201)],
                clue_ranks: vec![ClueRank::new(255)],
            },
            IdentitySpec {
                suit: SuitId::new(7),
                rank: CardRank::new(4),
                copies: 3,
                clue_colors: vec![ClueColorId::new(200)],
                clue_ranks: vec![ClueRank::new(0)],
            },
            IdentitySpec {
                suit: SuitId::new(9),
                rank: CardRank::new(2),
                copies: 3,
                clue_colors: vec![ClueColorId::new(200)],
                clue_ranks: vec![ClueRank::new(0)],
            },
            IdentitySpec {
                suit: SuitId::new(9),
                rank: CardRank::new(4),
                copies: 3,
                clue_colors: vec![ClueColorId::new(200), ClueColorId::new(201)],
                clue_ranks: vec![ClueRank::new(255)],
            },
        ],
    )
    .unwrap()
}

fn assert_history_replays(mut replayed: FullState, state: &FullState) {
    for entry in state.history() {
        let action = match &entry.event {
            GameEvent::Clued { target, clue, .. } => PhysicalAction::Clue {
                target: *target,
                clue: *clue,
            },
            GameEvent::Played { card, .. } => PhysicalAction::Play(*card),
            GameEvent::Discarded { card, .. } => PhysicalAction::Discard(*card),
            GameEvent::Drew { .. } => continue,
        };
        replayed.apply_physical(action).unwrap();
    }
    assert_eq!(&replayed, state);
}

#[test]
fn generic_color_rank_apply_and_physical_history_round_trip() {
    let spec = custom_spec();
    let c = |s, r| Card::new(SuitId::new(s), CardRank::new(r));
    let deck = deck_for(
        &spec,
        &[
            c(7, 2),
            c(7, 4),
            c(9, 2),
            c(9, 4),
            c(7, 2),
            c(7, 2),
            c(9, 2),
            c(7, 4),
            c(9, 4),
            c(9, 2),
        ],
    );
    let initial = FullState::new_variant(2, spec.clone(), deck.clone()).unwrap();
    let expected = (0..5)
        .map(|id| PhysicalAction::Play(CardId::new(id)))
        .chain([
            clue(1, color(200)),
            clue(1, color(201)),
            clue(1, rank(0)),
            clue(1, rank(255)),
        ])
        .collect::<Vec<_>>();
    assert_eq!(initial.legal_physical_actions(), expected);
    assert_eq!(
        initial
            .physical_view_for(PlayerId::new(0))
            .unwrap()
            .legal_physical_actions(),
        expected
    );
    let mut state = initial.clone();
    let actions = [
        clue(1, color(200)),
        clue(0, rank(255)),
        PhysicalAction::Discard(CardId::new(0)),
    ];
    for action in actions {
        assert_eq!(state.apply_physical(action).unwrap().action, action);
    }
    assert_eq!(state.clue_tokens(), 7);
    assert_eq!(state.turn(), 3);
    assert!(
        matches!(&state.history()[0].event, GameEvent::Clued { clue: value, touched, untouched, .. }
        if *value == color(200) && *touched == [6, 7, 8, 9].map(CardId::new) && *untouched == [CardId::new(5)])
    );
    assert!(
        matches!(&state.history()[1].event, GameEvent::Clued { clue: value, touched, untouched, .. }
        if *value == rank(255) && *touched == [0, 3, 4].map(CardId::new) && *untouched == [1, 2].map(CardId::new))
    );
    assert!(state.standard_history().is_err());
    assert!(state.view_for(PlayerId::new(0)).is_none());
    assert_history_replays(initial, &state);
    for observer in [PlayerId::new(0), PlayerId::new(1)] {
        let view = state.physical_view_for(observer).unwrap();
        assert!(
            view.hands[observer.index()]
                .iter()
                .all(|card| card.identity.is_none())
        );
        for card in view.hands.iter().flatten() {
            let id = spec
                .identities()
                .id_for(
                    state.card(card.id).unwrap().suit,
                    state.card(card.id).unwrap().rank,
                )
                .unwrap();
            assert_ne!(card.clues.identity_mask_in(&spec) & (1 << id.value()), 0);
        }
        assert!(
            view.hands[1][1]
                .clues
                .has_positive_physical_clue(color(200))
        );
        assert!(
            view.hands[1][0]
                .clues
                .has_negative_physical_clue(color(200))
        );
        assert!(
            matches!(&view.history.last().unwrap().event, ObservedEvent::Drew { player, identity, .. }
            if identity.is_none() == (*player == observer))
        );
        let world = WorldTemplate::new(&view)
            .unwrap()
            .instantiate(deck.clone())
            .unwrap();
        assert_eq!(world, state);
        assert_eq!(world.physical_view_for(observer).unwrap(), view);
    }
    state.validate().unwrap();
}

#[test]
fn catalog_clue_legality_touching_and_facts_use_physical_rules() {
    // Target ids 5 and 8 are the special suit; id 6 is red 2.
    for (variant, red, two, extra_color, one) in [
        (
            VariantId::Rainbow6Suits,
            vec![5, 6, 8],
            vec![5, 6],
            false,
            vec![],
        ),
        (VariantId::White6Suits, vec![6], vec![5, 6], false, vec![]),
        (
            VariantId::Pink6Suits,
            vec![6],
            vec![5, 6, 8],
            true,
            vec![5, 8],
        ),
        (VariantId::Brown6Suits, vec![6], vec![6], true, vec![]),
    ] {
        let spec = VariantSpec::from_catalog(variant);
        let mut prefix = vec![Card::new(Suit::Red, Rank::One); 3];
        prefix.extend([
            Card::new(Suit::Red, Rank::Three),
            Card::new(Suit::Red, Rank::Four),
        ]);
        prefix.extend([
            Card::new(SuitId::new(5), Rank::Two),
            Card::new(Suit::Red, Rank::Two),
            Card::new(Suit::Blue, Rank::Three),
            Card::new(SuitId::new(5), Rank::Four),
            Card::new(Suit::Yellow, Rank::Four),
        ]);
        let initial = FullState::new_variant(2, spec.clone(), deck_for(&spec, &prefix)).unwrap();
        let legal = initial.legal_physical_actions();
        assert_eq!(
            initial
                .physical_view_for(PlayerId::new(0))
                .unwrap()
                .legal_physical_actions(),
            legal
        );
        assert_eq!(legal.contains(&clue(1, color(5))), extra_color);
        assert_eq!(legal.contains(&clue(1, rank(1))), !one.is_empty());
        let mut cases = vec![(color(0), red), (rank(2), two)];
        if extra_color {
            cases.push((color(5), vec![5, 8]));
        }
        if !one.is_empty() {
            cases.push((rank(1), one));
        }
        for (generic, expected_ids) in cases {
            let mut state = initial.clone();
            state.apply_physical(clue(1, generic)).unwrap();
            let expected = expected_ids
                .into_iter()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let GameEvent::Clued {
                touched, untouched, ..
            } = &state.history()[0].event
            else {
                panic!("clue event")
            };
            assert_eq!(*touched, expected, "{variant:?}: {generic:?}");
            assert_eq!(touched.len() + untouched.len(), 5);
            let view = state.physical_view_for(PlayerId::new(1)).unwrap();
            for card in &view.hands[1] {
                let positive = expected.contains(&card.id);
                assert_eq!(card.clues.has_positive_physical_clue(generic), positive);
                assert_eq!(card.clues.has_negative_physical_clue(generic), !positive);
                let expected_mask =
                    spec.identities()
                        .identities()
                        .iter()
                        .fold(0, |mask, identity| {
                            mask | (u64::from(spec.touches(identity.id, generic) == positive)
                                << identity.id.value())
                        });
                assert_eq!(card.clues.identity_mask_in(&spec), expected_mask);
            }
            state.validate().unwrap();
        }
    }
}

#[test]
fn multiple_rainbow_colors_intersect_physical_constraints() {
    let spec = VariantSpec::from_catalog(VariantId::Rainbow6Suits);
    let mut prefix = vec![Card::new(Suit::Red, Rank::One); 3];
    prefix.extend([
        Card::new(Suit::Red, Rank::Three),
        Card::new(Suit::Red, Rank::Four),
    ]);
    prefix.extend([
        Card::new(SuitId::new(5), Rank::Two),
        Card::new(Suit::Red, Rank::Two),
        Card::new(Suit::Blue, Rank::Three),
        Card::new(SuitId::new(5), Rank::Four),
        Card::new(Suit::Yellow, Rank::Four),
    ]);
    let mut state = FullState::new_variant(2, spec.clone(), deck_for(&spec, &prefix)).unwrap();
    state.apply_physical(clue(1, color(0))).unwrap();
    state.apply_physical(clue(0, color(0))).unwrap();
    state.apply_physical(clue(1, color(3))).unwrap();
    let view = state.physical_view_for(PlayerId::new(1)).unwrap();
    let expected = spec
        .identities()
        .identities()
        .iter()
        .filter(|id| id.suit == SuitId::new(5))
        .fold(0, |mask, id| mask | (1 << id.id.value()));
    assert_eq!(view.hands[1][0].clues.identity_mask_in(&spec), expected);
    assert_eq!(view.hands[1][3].clues.identity_mask_in(&spec), expected);
    assert_eq!(
        view.hands[1][1].clues.identity_mask_in(&spec),
        spec.identities().clue_mask(color(0)) & !spec.identities().clue_mask(color(3))
    );
}

#[test]
fn illegal_physical_actions_reject_without_mutation() {
    let initial = FullState::new_standard(2, standard_deck()).unwrap();
    for (action, error) in [
        (clue(1, color(5)), RuleError::InvalidClue(color(5))),
        (clue(1, rank(0)), RuleError::InvalidClue(rank(0))),
        (clue(1, rank(6)), RuleError::InvalidClue(rank(6))),
        (clue(1, color(4)), RuleError::ClueTouchesNoCards),
        (clue(0, color(0)), RuleError::CannotClueSelf),
        (
            clue(2, color(0)),
            RuleError::InvalidPlayer(PlayerId::new(2)),
        ),
        (
            PhysicalAction::Play(CardId::new(5)),
            RuleError::CardNotInCurrentHand(CardId::new(5)),
        ),
        (
            PhysicalAction::Discard(CardId::new(0)),
            RuleError::DiscardAtMaximumClues,
        ),
    ] {
        let mut state = initial.clone();
        assert_eq!(state.apply_physical(action), Err(error));
        assert_eq!(state, initial);
    }
    let mut state = initial;
    for _ in 0..8 {
        let action = state
            .legal_physical_actions()
            .into_iter()
            .find(|a| matches!(a, PhysicalAction::Clue { .. }))
            .unwrap();
        state.apply_physical(action).unwrap();
    }
    assert!(
        state
            .legal_physical_actions()
            .iter()
            .all(|a| !matches!(a, PhysicalAction::Clue { .. }))
    );
    assert_eq!(
        state
            .physical_view_for(state.current_player())
            .unwrap()
            .legal_physical_actions(),
        state.legal_physical_actions()
    );
    let before = state.clone();
    assert_eq!(
        state.apply_physical(clue(1, color(0))),
        Err(RuleError::NoClueTokens)
    );
    assert_eq!(state, before);
}

#[test]
fn terminal_physical_state_has_no_actions_and_rejects_apply() {
    let mut state = FullState::new_standard(2, standard_deck()).unwrap();
    for _ in 0..3 {
        let id = *state
            .hand(state.current_player())
            .unwrap()
            .iter()
            .find(|id| state.card(**id).unwrap().rank != Rank::One)
            .unwrap();
        state.apply_physical(PhysicalAction::Play(id)).unwrap();
    }
    assert!(state.is_terminal());
    assert_eq!(state.legal_physical_actions(), Vec::<PhysicalAction>::new());
    assert_eq!(
        state
            .physical_view_for(state.current_player())
            .unwrap()
            .legal_physical_actions(),
        Vec::<PhysicalAction>::new()
    );
    let before = state.clone();
    assert_eq!(
        state.apply_physical(clue(1, color(0))),
        Err(RuleError::GameAlreadyFinished)
    );
    assert_eq!(state, before);
}
