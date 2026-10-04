use hanabi_core::{
    Action, Card, CardRank, Clue, ClueColorId, ClueFacts, ClueRank, FullState, GenericClue,
    IdentityId, IdentitySpec, PhysicalAction, PlayerId, Rank, Suit, SuitId, VariantSpec,
    standard_deck,
};
use hanabi_search::{
    BeliefConstraints, ConventionAgnosticPolicy, IdentitySet, InformationSet, InformationSetError,
    LogicalDeductions, WorldCount,
};

fn deck(variant: &VariantSpec) -> Vec<Card> {
    variant
        .identities()
        .identities()
        .iter()
        .flat_map(|identity| {
            std::iter::repeat_n(
                Card::new(identity.suit, identity.rank),
                usize::from(identity.copies),
            )
        })
        .collect()
}

fn custom_variant() -> VariantSpec {
    VariantSpec::new(
        "test physical rules",
        vec![SuitId::new(17), SuitId::new(250)],
        vec![CardRank::new(10), CardRank::new(200)],
        vec![ClueColorId::new(199), ClueColorId::new(2)],
        vec![ClueRank::new(255), ClueRank::new(1)],
        [(250, 200, 1), (17, 10, 4), (250, 10, 3), (17, 200, 4)]
            .into_iter()
            .enumerate()
            .map(|(index, (suit, rank, copies))| IdentitySpec {
                suit: SuitId::new(suit),
                rank: CardRank::new(rank),
                copies,
                clue_colors: match index {
                    0 => vec![ClueColorId::new(199), ClueColorId::new(2)],
                    1 => vec![ClueColorId::new(199)],
                    2 => vec![],
                    _ => vec![ClueColorId::new(2)],
                },
                clue_ranks: vec![ClueRank::new(if rank == 200 { 255 } else { 1 })],
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn arbitrary_ids_multiplicities_and_worlds_use_the_public_universe() {
    let variant = custom_variant();
    let state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    assert_eq!(view.variant, variant);
    assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
    let information = InformationSet::new(&view).unwrap();
    // P1 sees all three copies of ID 2, but only two of the four copies of ID 3.
    let expected = IdentitySet::all_in(variant.identities())
        .without(IdentitySet::singleton_id(IdentityId::new(2)));
    for card in &view.hands[0] {
        assert_eq!(information.possible_identities(card.id), Some(expected));
    }
    assert_eq!(
        expected.iter_in(variant.identities()).collect::<Vec<_>>(),
        vec![
            Card::new(SuitId::new(250), CardRank::new(200)),
            Card::new(SuitId::new(17), CardRank::new(10)),
            Card::new(SuitId::new(17), CardRank::new(200))
        ]
    );
    // Seven hidden locations contain counts [1, 4, 0, 2]: 7! / (4! * 2!).
    assert_eq!(
        information.world_count_up_to(&BeliefConstraints::default(), 200),
        WorldCount::Exact(105)
    );
    let mut unique = std::collections::HashSet::new();
    let visited = information
        .visit_worlds(&BeliefConstraints::default(), 200, |world| {
            world.validate().unwrap();
            assert_eq!(world.view_for(view.observer), Some(view.clone()));
            assert!(unique.insert(format!("{world:?}")));
        })
        .unwrap();
    assert_eq!(visited, 105);
}

#[test]
fn generic_clues_filter_positive_and_negative_touches_without_conventions() {
    let variant = custom_variant();
    let all = IdentitySet::all_in(variant.identities());
    // Legacy clue facts also resolve through the variant: color 2 touches
    // physical suits 250 and 17, and clue rank 1 touches physical rank 10.
    let mut facts = ClueFacts::default();
    facts.add_positive_clue(Clue::Suit(Suit::Green));
    assert_eq!(facts.identity_mask_in(&variant), 0b1001);
    let mut rank_facts = ClueFacts::default();
    rank_facts.add_positive_clue(Clue::Rank(Rank::One));
    assert_eq!(rank_facts.identity_mask_in(&variant), 0b0110);
    let color = GenericClue::Color(ClueColorId::new(199));
    let positive = all.filter_clue(&variant, color, true);
    let negative = all.filter_clue(&variant, color, false);
    assert_eq!(
        positive.iter_ids().collect::<Vec<_>>(),
        vec![IdentityId::new(0), IdentityId::new(1)]
    );
    assert_eq!(
        negative.iter_ids().collect::<Vec<_>>(),
        vec![IdentityId::new(2), IdentityId::new(3)]
    );
    assert_eq!(positive.union(negative), all);
    assert!(positive.intersection(negative).is_empty());
    assert_eq!(
        positive.filter_clue(&variant, GenericClue::Rank(ClueRank::new(255)), true),
        IdentitySet::singleton_id(IdentityId::new(0))
    );
    assert!(
        all.filter_clue(&variant, GenericClue::Color(ClueColorId::new(254)), true)
            .is_empty()
    );
    let state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    let information = InformationSet::new(&view).unwrap();
    let first = view.hands[0][0].id;
    let belief = BeliefConstraints {
        constraints: vec![(first, positive)],
        factors: vec![],
    };
    assert_eq!(
        information.world_count_up_to(&belief, 100),
        WorldCount::Exact(75)
    );
    information
        .visit_worlds(&belief, 100, |world| {
            let identity = world.card(first).unwrap();
            let id = variant
                .identities()
                .id_for(identity.suit, identity.rank)
                .unwrap();
            assert!(variant.touches(id, color));
        })
        .unwrap();
}

#[test]
fn reordered_standard_cards_do_not_use_card_index_for_counts_or_clues() {
    let standard = VariantSpec::no_variant();
    let variant = VariantSpec::new(
        "reordered standard identities",
        standard.stack_suits().to_vec(),
        standard.card_ranks().to_vec(),
        standard.clue_colors().to_vec(),
        standard.clue_ranks().to_vec(),
        standard
            .identities()
            .identities()
            .iter()
            .rev()
            .map(|identity| IdentitySpec {
                suit: identity.suit,
                rank: identity.rank,
                copies: identity.copies,
                clue_colors: identity.clue_colors.clone(),
                clue_ranks: identity.clue_ranks.clone(),
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(
        variant.identities().id_for(Suit::Purple, Rank::Five),
        Some(IdentityId::new(0))
    );
    assert_eq!(Card::new(Suit::Purple, Rank::Five).index(), 24);
    let mut state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    state
        .apply(Action::Clue {
            target: PlayerId::new(1),
            clue: Clue::Rank(Rank::One),
        })
        .unwrap();
    state
        .apply(Action::Clue {
            target: PlayerId::new(0),
            clue: Clue::Suit(Suit::Purple),
        })
        .unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    let information = InformationSet::new(&view).unwrap();
    for card in &view.hands[0] {
        assert_eq!(
            information
                .possible_identities(card.id)
                .unwrap()
                .iter_ids()
                .collect::<Vec<_>>(),
            [0, 1, 2].map(IdentityId::new)
        );
    }
}

#[test]
fn all_64_identities_survive_masks_and_feasibility() {
    let ranks = (1..=64).map(CardRank::new).collect::<Vec<_>>();
    let variant = VariantSpec::new(
        "64 identities",
        vec![Suit::Red],
        ranks.clone(),
        vec![],
        vec![],
        ranks
            .into_iter()
            .map(|rank| IdentitySpec {
                suit: Suit::Red,
                rank,
                copies: 1,
                clue_colors: vec![],
                clue_ranks: vec![],
            })
            .collect(),
    )
    .unwrap();
    let all = IdentitySet::all_in(variant.identities());
    assert_eq!(all.len(), 64);
    assert!(all.contains_id(IdentityId::new(63)));
    assert_eq!(
        all.iter_in(variant.identities()).last(),
        Some(Card::new(Suit::Red, CardRank::new(64)))
    );
    assert!(!all.contains_id(IdentityId::new(64)));
    let state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    let information = InformationSet::new(&view).unwrap();
    let possibilities = information
        .possible_identities(view.hands[0][0].id)
        .unwrap();
    assert_eq!(possibilities.len(), 59);
    assert!(possibilities.contains_id(IdentityId::new(63)));
    assert_eq!(
        information.world_count_up_to(&BeliefConstraints::default(), 1),
        WorldCount::LowerBound(2)
    );
}

#[test]
fn variant_copy_capacities_can_exceed_u8_without_overflow() {
    let variant = VariantSpec::new(
        "large counts",
        vec![SuitId::new(17), SuitId::new(18)],
        vec![CardRank::new(42)],
        vec![],
        vec![],
        [17, 18]
            .map(|suit| IdentitySpec {
                suit: SuitId::new(suit),
                rank: CardRank::new(42),
                copies: 255,
                clue_colors: vec![],
                clue_ranks: vec![],
            })
            .to_vec(),
    )
    .unwrap();
    let state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let deductions = LogicalDeductions::new(state.view_for(PlayerId::new(0)).unwrap()).unwrap();
    for card in state.hand(PlayerId::new(0)).unwrap() {
        assert_eq!(
            deductions.possible_identities(*card),
            Some(IdentitySet::all_in(variant.identities()))
        );
    }
}

#[test]
fn rejects_visible_identities_outside_the_active_universe() {
    let variant = custom_variant();
    let state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let mut view = state.view_for(PlayerId::new(0)).unwrap();
    let alien = Card::new(Suit::Red, Rank::One);
    view.hands[1][0].identity = Some(alien);
    assert_eq!(
        InformationSet::new(&view),
        Err(InformationSetError::UnknownIdentity(alien))
    );
}

#[test]
fn no_variant_helpers_and_direct_masks_preserve_standard_behavior() {
    let variant = VariantSpec::no_variant();
    let all = IdentitySet::all_in(variant.identities());
    assert_eq!(all, IdentitySet::all());
    assert_eq!(
        all.iter_in(variant.identities()).collect::<Vec<_>>(),
        all.iter().collect::<Vec<_>>()
    );
    for identity in variant.identities().identities() {
        let card = Card::new(identity.suit, identity.rank);
        assert_eq!(
            IdentitySet::singleton_id(identity.id),
            IdentitySet::singleton(card)
        );
        assert_eq!(all.contains_id(identity.id), all.contains(card));
    }
    for clue in Suit::ALL
        .map(Clue::Suit)
        .into_iter()
        .chain(Rank::ALL.map(Clue::Rank))
    {
        for positive in [false, true] {
            let mut facts = ClueFacts::default();
            if positive {
                facts.add_positive_clue(clue);
            } else {
                facts.add_negative_clue(clue);
            }
            assert_eq!(
                facts.identity_mask_in(&variant),
                u64::from(facts.identity_mask())
            );
        }
    }
}

#[test]
fn six_player_search_boundary_and_baseline_smoke() {
    let state = FullState::new_standard(6, standard_deck()).unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    let information = InformationSet::new(&view).unwrap();
    assert_eq!(view.hands.len(), 6);
    assert_eq!(view.hands[0].len(), 3);
    assert_eq!(
        information.world_count_up_to(&BeliefConstraints::default(), 1),
        WorldCount::LowerBound(2)
    );
    let action = ConventionAgnosticPolicy
        .select_action(information.deductions())
        .unwrap();
    assert!(state.legal_actions().contains(&action));
}

#[test]
fn physical_generic_history_and_multiplicities_enter_logical_search() {
    let variant = custom_variant();
    let mut state = FullState::new_variant(2, variant.clone(), deck(&variant)).unwrap();
    let clues = [
        (PlayerId::new(1), GenericClue::Rank(ClueRank::new(1))),
        (PlayerId::new(0), GenericClue::Color(ClueColorId::new(199))),
    ];
    for (target, clue) in clues {
        state
            .apply_physical(PhysicalAction::Clue { target, clue })
            .unwrap();
    }
    let view: hanabi_core::PlayerView<GenericClue> =
        state.physical_view_for(PlayerId::new(0)).unwrap();
    let deductions: LogicalDeductions<GenericClue> = LogicalDeductions::new(view.clone()).unwrap();
    let information: InformationSet<GenericClue> = InformationSet::new(&view).unwrap();
    assert_eq!(deductions.view(), &view);
    assert_eq!(information.view(), &view);
    assert_eq!(information.deductions(), &deductions);
    assert_eq!(deductions.clone(), deductions);
    assert_eq!(information.clone(), information);
    for (entry, (_, original)) in information.view().history.iter().zip(clues) {
        assert!(
            matches!(entry.event, hanabi_core::ObservedEvent::Clued { clue, .. } if clue == original)
        );
    }
    assert_eq!(information.view().history.len(), clues.len());
    // Visible counts leave [1, 4, 0, 2]. The color clue restricts all five
    // own cards to IDs 0 and 1, so only the location of ID 0 varies.
    let expected = IdentitySet::singleton_id(IdentityId::new(0))
        .union(IdentitySet::singleton_id(IdentityId::new(1)));
    for card in &deductions.view().hands[0] {
        assert!(card.identity.is_none());
        assert_eq!(deductions.possible_identities(card.id), Some(expected));
        assert_eq!(information.possible_identities(card.id), Some(expected));
    }
    let belief = BeliefConstraints::default();
    assert_eq!(
        information.world_count_up_to(&belief, 10),
        WorldCount::Exact(5)
    );
    assert_eq!(
        information
            .visit_worlds(&belief, 10, |world| {
                world.validate().unwrap();
                assert_eq!(world.physical_view_for(view.observer), Some(view.clone()));
            })
            .unwrap(),
        5
    );
    let constrained = BeliefConstraints {
        constraints: vec![(
            view.hands[0][0].id,
            IdentitySet::singleton_id(IdentityId::new(0)),
        )],
        factors: vec![],
    };
    assert_eq!(
        information.world_count_up_to(&constrained, 10),
        WorldCount::Exact(1)
    );
}

fn sixth_suit_variant(prism: bool) -> VariantSpec {
    let standard = VariantSpec::no_variant();
    let sixth = SuitId::new(5);
    let mut identities = standard
        .identities()
        .identities()
        .iter()
        .map(|identity| IdentitySpec {
            suit: identity.suit,
            rank: identity.rank,
            copies: identity.copies,
            clue_colors: identity.clue_colors.clone(),
            clue_ranks: identity.clue_ranks.clone(),
        })
        .collect::<Vec<_>>();
    identities.extend(Rank::ALL.into_iter().map(|rank| IdentitySpec {
        suit: sixth,
        rank,
        copies: if rank == Rank::One {
            3
        } else if rank == Rank::Five {
            1
        } else {
            2
        },
        // Rainbow touches every color; Prism touches the color selected by rank.
        clue_colors: if prism {
            vec![standard.clue_colors()[usize::from(rank.number() - 1)]]
        } else {
            standard.clue_colors().to_vec()
        },
        clue_ranks: vec![ClueRank::new(rank.number())],
    }));
    let mut suits = standard.stack_suits().to_vec();
    suits.push(sixth);
    VariantSpec::new(
        if prism {
            "test Prism (6 suits)"
        } else {
            "test Rainbow (6 suits)"
        },
        suits,
        standard.card_ranks().to_vec(),
        standard.clue_colors().to_vec(),
        standard.clue_ranks().to_vec(),
        identities,
    )
    .unwrap()
}

fn physical_sixth_suit_view(prism: bool) -> hanabi_core::PlayerView<GenericClue> {
    let variant = sixth_suit_variant(prism);
    let sixth = SuitId::new(5);
    let prefix = [
        Card::new(sixth, Rank::One),
        Card::new(sixth, Rank::Two),
        Card::new(Suit::Red, Rank::One),
        Card::new(Suit::Yellow, Rank::Two),
        Card::new(Suit::Green, Rank::Three),
        Card::new(Suit::Yellow, Rank::One),
        Card::new(Suit::Purple, Rank::Two),
        Card::new(Suit::Red, Rank::Three),
        Card::new(Suit::Blue, Rank::Four),
        Card::new(Suit::Green, Rank::Five),
    ];
    let mut cards = deck(&variant);
    for (index, wanted) in prefix.iter().enumerate() {
        let found = cards[index..]
            .iter()
            .position(|card| card == wanted)
            .unwrap()
            + index;
        cards.swap(index, found);
    }
    let mut state = FullState::new_variant(2, variant, cards).unwrap();
    for (target, clue) in [
        (PlayerId::new(1), GenericClue::Rank(ClueRank::new(1))),
        (PlayerId::new(0), GenericClue::Color(ClueColorId::new(0))),
        (PlayerId::new(1), GenericClue::Rank(ClueRank::new(2))),
        (PlayerId::new(0), GenericClue::Color(ClueColorId::new(1))),
    ] {
        state
            .apply_physical(PhysicalAction::Clue { target, clue })
            .unwrap();
    }
    state.physical_view_for(PlayerId::new(0)).unwrap()
}

#[test]
fn rainbow_color_touches_preserve_sixth_suit_possibilities() {
    let view = physical_sixth_suit_view(false);
    let information = InformationSet::new(&view).unwrap();
    let universe = view.variant.identities();
    assert_eq!(IdentitySet::all_in(universe).len(), 30);
    let rainbow = Rank::ALL
        .into_iter()
        .fold(IdentitySet::default(), |set, rank| {
            set.union(IdentitySet::singleton_id(
                universe.id_for(SuitId::new(5), rank).unwrap(),
            ))
        });
    // Both Rainbow cards were touched by both color clues, regardless of rank.
    for card in &view.hands[0][..2] {
        assert!(card.identity.is_none());
        assert_eq!(information.possible_identities(card.id), Some(rainbow));
        assert_eq!(
            information.deductions().possible_identities(card.id),
            Some(rainbow)
        );
    }
    // Negative touches rule Rainbow out of each ordinary card.
    for card in &view.hands[0][2..] {
        assert!(card.identity.is_none());
        let possible = information.possible_identities(card.id).unwrap();
        assert!(possible.intersection(rainbow).is_empty());
        assert_eq!(
            possible.intersection(IdentitySet::all_in(universe)),
            possible
        );
    }
    assert_eq!(information.view(), &view);
    assert_eq!(
        information.world_count_up_to(&BeliefConstraints::default(), 1),
        WorldCount::LowerBound(2)
    );
}

#[test]
fn prism_color_touches_follow_rank_in_the_active_universe() {
    let view = physical_sixth_suit_view(true);
    let deductions = LogicalDeductions::new(view.clone()).unwrap();
    let information = InformationSet::new(&view).unwrap();
    let universe = view.variant.identities();
    let prism_one = universe.id_for(SuitId::new(5), Rank::One).unwrap();
    let prism_two = universe.id_for(SuitId::new(5), Rank::Two).unwrap();
    for (slot, expected_prism, ordinary_suit) in [
        (0, prism_one, Suit::Red),
        (1, prism_two, Suit::Yellow),
        (2, prism_one, Suit::Red),
        (3, prism_two, Suit::Yellow),
    ] {
        let card = &deductions.view().hands[0][slot];
        assert!(card.identity.is_none());
        let possible = deductions.possible_identities(card.id).unwrap();
        let expected =
            Rank::ALL
                .into_iter()
                .fold(IdentitySet::singleton_id(expected_prism), |set, rank| {
                    set.union(IdentitySet::singleton_id(
                        universe.id_for(ordinary_suit, rank).unwrap(),
                    ))
                });
        assert_eq!(possible, expected);
        assert_eq!(information.possible_identities(card.id), Some(expected));
    }
    // An untouched card can still be Prism 3, 4, or 5, but not Prism 1 or 2.
    let untouched = deductions.possible_identities(view.hands[0][4].id).unwrap();
    assert!(!untouched.contains_id(prism_one));
    assert!(!untouched.contains_id(prism_two));
    for rank in [Rank::Three, Rank::Four, Rank::Five] {
        assert!(untouched.contains_id(universe.id_for(SuitId::new(5), rank).unwrap()));
    }
    assert_eq!(information.view(), &view);
}

#[test]
fn physical_no_variant_deductions_match_the_legacy_default() {
    let mut state = FullState::new_standard(2, standard_deck()).unwrap();
    state
        .apply(Action::Clue {
            target: PlayerId::new(1),
            clue: Clue::Suit(Suit::Red),
        })
        .unwrap();
    state
        .apply(Action::Clue {
            target: PlayerId::new(0),
            clue: Clue::Rank(Rank::One),
        })
        .unwrap();
    let legacy_view: hanabi_core::PlayerView = state.view_for(PlayerId::new(0)).unwrap();
    let generic_view = state.physical_view_for(PlayerId::new(0)).unwrap();
    let legacy: InformationSet = InformationSet::new(&legacy_view).unwrap();
    let legacy_deductions: LogicalDeductions = LogicalDeductions::new(legacy_view).unwrap();
    let generic = InformationSet::new(&generic_view).unwrap();
    assert_eq!(legacy.deductions(), &legacy_deductions);
    assert_eq!(
        legacy_deductions.unknown_hand_cards(),
        generic.deductions().unknown_hand_cards()
    );
    for card in &generic.view().hands[0] {
        assert!(card.identity.is_none());
        assert_eq!(
            legacy.possible_identities(card.id),
            generic.possible_identities(card.id)
        );
    }
    let belief = BeliefConstraints::default();
    assert_eq!(
        legacy.world_count_up_to(&belief, 10),
        generic.world_count_up_to(&belief, 10)
    );
}
