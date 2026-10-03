use hanabi_core::{
    Action, Card, CardRank, Clue, ClueColorId, ClueFacts, ClueRank, FullState, GenericClue,
    IdentityId, IdentitySpec, PlayerId, Rank, Suit, SuitId, VariantSpec, standard_deck,
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
