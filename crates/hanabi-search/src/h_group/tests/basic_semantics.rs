// Artificial snapshot inputs verify the generic algorithm contracts requested
// for M4-V1. They assert no strategy choices or invented replay interpretations.
use super::*;
use hanabi_core::{GenericClue, VariantId, VariantSpec};

fn snapshot(variant: VariantSpec) -> PlayerView {
    let mut view = FullState::new_standard(3, standard_deck())
        .unwrap()
        .view_for(PlayerId::new(0))
        .unwrap();
    view.play_stacks = vec![Vec::new(); variant.stack_suits().len()];
    view.variant = variant;
    for card in view.hands.iter_mut().flatten() {
        card.identity = None;
    }
    view
}

fn domain(variant: &VariantSpec, suit: u8, rank: u8) -> IdentitySet {
    IdentitySet::singleton_id(
        variant
            .identities()
            .id_for(Suit::new(suit), Rank::new(rank))
            .unwrap(),
    )
}

fn layouts(view: &PlayerView) -> Vec<Vec<CardId>> {
    view.hands
        .iter()
        .map(|hand| hand.iter().map(|card| card.id).collect::<Vec<_>>())
        .collect()
}

// A legal physical prefix checks history bookkeeping, not convention strategy.
#[test]
fn contract_history_reducer_successful_active_last_suit_play() {
    use hanabi_core::PhysicalAction;
    for variant_id in [VariantId::NoVariant, VariantId::White6Suits] {
        let variant = VariantSpec::from_catalog(variant_id);
        let suit = *variant.stack_suits().last().unwrap();
        let mut deck = variant
            .identities()
            .identities()
            .iter()
            .flat_map(|identity| {
                std::iter::repeat_n(
                    Card::new(identity.suit, identity.rank),
                    usize::from(identity.copies),
                )
            })
            .collect::<Vec<_>>();
        let first = deck
            .iter()
            .position(|card| card.suit == suit && card.rank == Rank::One)
            .unwrap();
        deck.swap(0, first);
        let mut state = FullState::new_variant(3, variant, deck).unwrap();
        state
            .apply_physical(PhysicalAction::Play(CardId::new(0)))
            .unwrap();
        let view = state
            .physical_view_for(PlayerId::new(0))
            .unwrap()
            .try_standard()
            .unwrap();
        let deductions = LogicalDeductions::new(view.clone()).unwrap();
        let replay = replay_h_group_inner(
            &deductions,
            HGroupProfile::Level(HGroupLevel::Level1),
            PerspectiveDepth::ObserverOnly,
            false,
        );
        assert_eq!(replay.hands, layouts(&view));
        let facts = vec![ClueFacts::default(); view.variant.identities().deck_size()];
        let subjective = super::super::prospective::subjective_view_before_action(
            &view,
            view.observer,
            &view.history,
            &layouts(&view),
            &facts,
            view.deck_size,
        );
        assert_eq!(subjective.variant, view.variant);
        assert_eq!(subjective.play_stacks, view.play_stacks);
        assert_eq!(
            super::super::identity::connection_heights(&view, None).last(),
            Some(&1)
        );
    }
}

// Artificial inputs check duplicate accounting across the active sixth suit,
// not a White convention meaning or an optimal clue.
#[test]
fn contract_admission_good_touch_preserves_white_identity() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::White6Suits));
    let white = Card::new(Suit::new(5), Rank::One);
    view.hands[1][0].identity = Some(white);
    view.hands[1][1].identity = Some(white);
    let touched = [view.hands[1][0].id, view.hands[1][1].id];
    let empty = CardSet::default();
    let context = super::super::admission::GoodTouchContext {
        view: &view,
        newly_touched: &touched[..1],
        clue: None,
        explicitly_clued: &empty,
        fixed_cards: &empty,
        convention_cards: &[],
    };
    assert!(super::super::admission::good_touch(context));
    assert!(!super::super::admission::good_touch(
        super::super::admission::GoodTouchContext {
            newly_touched: &touched,
            ..context
        }
    ));
    assert_eq!(
        super::super::admission::minimum_clue_value(&view, &[], touched),
        1
    );
}

#[test]
fn contract_basic_good_touch_no_variant_matches_legacy_usefulness_and_claims() {
    let mut view = snapshot(VariantSpec::no_variant());
    let focus = view.hands[0][0].id;
    let duplicate = view.hands[1][0].id;
    let visible = Card::new(Suit::Red, Rank::Three);
    view.hands[1][0].identity = Some(visible);
    let hands = layouts(&view);
    let gotten = [duplicate].into_iter().collect();
    for heights in [[0; 5], [1, 2, 3, 4, 5], [5; 5]] {
        for exhausted in [
            None,
            Some(Card::new(Suit::Red, Rank::Two)),
            Some(Card::new(Suit::Blue, Rank::One)),
        ] {
            let mut discarded = [0; 25];
            if let Some(card) = exhausted {
                discarded[card.index()] = usize::from(card.rank.copies());
            }
            let expected = IdentitySet::from_mask(
                IdentitySet::all()
                    .iter()
                    .filter(|identity| {
                        identity.rank.number() > heights[identity.suit.index()]
                            && Rank::ALL
                                .into_iter()
                                .filter(|rank| {
                                    rank.number() > heights[identity.suit.index()]
                                        && rank.number() < identity.rank.number()
                                })
                                .all(|rank| {
                                    discarded[Card::new(identity.suit, rank).index()]
                                        < usize::from(rank.copies())
                                })
                            && *identity != visible
                    })
                    .fold(0, |mask, card| mask | (1 << card.index())),
            );
            assert_eq!(
                snapshot_good_touch_identities(
                    focus,
                    IdentitySet::all(),
                    &view,
                    &hands,
                    &gotten,
                    &heights.map(usize::from),
                    &discarded
                ),
                expected
            );
        }
    }
}

#[test]
fn contract_basic_good_touch_rainbow_and_pink_physical_touches_keep_usefulness_rules() {
    for variant_id in [VariantId::Rainbow6Suits, VariantId::Pink6Suits] {
        let mut view = snapshot(VariantSpec::from_catalog(variant_id));
        let focus = view.hands[0][0].id;
        let duplicate = view.hands[1][0].id;
        let one = domain(&view.variant, 5, 1);
        let three = domain(&view.variant, 5, 3);
        assert!(three.iter_ids().next().unwrap().value() >= 25);
        let clue = match variant_id {
            VariantId::Rainbow6Suits => GenericClue::Color(hanabi_core::ClueColorId::new(0)),
            _ => GenericClue::Rank(hanabi_core::ClueRank::new(1)),
        };
        let touched = one.union(three).filter_clue(&view.variant, clue, true);
        assert_eq!(touched, one.union(three));
        let hands = layouts(&view);
        let gotten = [duplicate].into_iter().collect();
        let heights = [0, 0, 0, 0, 0, 1];
        let discarded = [0; 30];
        assert_eq!(
            snapshot_good_touch_identities(
                focus, touched, &view, &hands, &gotten, &heights, &discarded
            ),
            three
        );
        view.hands[1][0].identity = Some(Card::new(Suit::new(5), Rank::Three));
        assert!(
            snapshot_good_touch_identities(
                focus, touched, &view, &hands, &gotten, &heights, &discarded
            )
            .is_empty()
        );
        // An unclaimed visible duplicate and a card outside the live layout
        // cannot remove the identity. Neither can the focus claim itself.
        for (live, claimed) in [
            (&hands[..], [focus].into_iter().collect()),
            (&hands[..1], gotten.clone()),
        ] {
            assert_eq!(
                snapshot_good_touch_identities(
                    focus, touched, &view, live, &claimed, &heights, &discarded
                ),
                three
            );
        }
        view.hands[1][0].identity = None;
        assert_eq!(
            snapshot_good_touch_identities(
                focus, touched, &view, &hands, &gotten, &heights, &discarded
            ),
            three
        );
    }
}

#[test]
fn contract_basic_good_touch_black_uses_one_copy_and_requires_each_missing_lower_identity() {
    for variant_id in [VariantId::Black6Suits, VariantId::White6Suits] {
        let view = snapshot(VariantSpec::from_catalog(variant_id));
        let hands = layouts(&view);
        let higher = domain(&view.variant, 5, 4);
        let lower = domain(&view.variant, 5, 2).iter_ids().next().unwrap();
        let copies = usize::from(view.variant.copies(lower).unwrap());
        assert_eq!(
            copies,
            if variant_id == VariantId::Black6Suits {
                1
            } else {
                2
            }
        );
        let mut discarded = [0; 30];
        let check = |heights: &[usize], discards: &[usize]| {
            snapshot_good_touch_identities(
                hands[0][0],
                higher,
                &view,
                &hands,
                &CardSet::default(),
                heights,
                discards,
            )
        };
        discarded[lower.index()] = copies - 1;
        assert_eq!(check(&[0; 6], &discarded), higher);
        discarded[lower.index()] = copies;
        assert!(check(&[0; 6], &discarded).is_empty());
        // An exhausted predecessor already on the stack is no longer required.
        assert_eq!(check(&[0, 0, 0, 0, 0, 2], &discarded), higher);
        assert!(check(&[0, 0, 0, 0, 0, 4], &discarded).is_empty());
    }
}

#[test]
fn contract_basic_good_touch_cannot_depend_on_hidden_simulator_truth() {
    let variant = VariantSpec::from_catalog(VariantId::Black6Suits);
    let mut deck = variant
        .identities()
        .identities()
        .iter()
        .flat_map(|identity| {
            std::iter::repeat_n(
                Card::new(identity.suit, identity.rank),
                usize::from(identity.copies),
            )
        })
        .collect::<Vec<_>>();
    let state = FullState::new_variant(3, variant.clone(), deck.clone()).unwrap();
    assert_ne!(deck[0], deck[3]);
    deck.swap(0, 3); // Change only the observer's hidden physical cards.
    let alternative = FullState::new_variant(3, variant.clone(), deck).unwrap();
    let view = state.physical_view_for(PlayerId::new(0)).unwrap();
    let other = alternative.physical_view_for(PlayerId::new(0)).unwrap();
    assert_eq!(view, other);
    let hands = view
        .hands
        .iter()
        .map(|hand| hand.iter().map(|card| card.id).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let gotten = hands[0].iter().copied().collect();
    let focus = hands[1][0];
    let all = IdentitySet::all_in(variant.identities());
    let check =
        |view| snapshot_good_touch_identities(focus, all, view, &hands, &gotten, &[0; 6], &[0; 30]);
    assert_eq!(check(&view), all);
    assert_eq!(check(&other), all);
}

#[test]
fn contract_basic_claims_map_visible_and_literal_sixth_suit_without_hidden_truth() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Black6Suits));
    let focus = view.hands[2][0].id;
    let visible = view.hands[1][0].id;
    let hidden = view.hands[0][0].id;
    let expected = domain(&view.variant, 5, 3);
    view.hands[1][0].identity = Some(Card::new(Suit::new(5), Rank::Three));
    let hands = layouts(&view);
    let mut facts = vec![ClueFacts::default(); 15];
    let gotten = [visible, hidden].into_iter().collect();
    let convention = ConventionFacts::default();
    let pending = ConnectionManager::default();
    let check = |view: &PlayerView, facts: &[ClueFacts]| {
        claimed_identities_at_clue(
            &view.variant,
            focus,
            &hands,
            &HistoricalView::new(view, 0),
            facts,
            &convention,
            &[],
            &gotten,
            &pending,
        )
    };
    assert_eq!(check(&view, &facts), expected);
    view.hands[1][0].identity = None;
    assert!(check(&view, &facts).is_empty());
    facts[hidden.index()]
        .record_physical(GenericClue::Color(hanabi_core::ClueColorId::new(5)), true);
    facts[hidden.index()].record_physical(GenericClue::Rank(hanabi_core::ClueRank::new(3)), true);
    assert_eq!(check(&view, &facts), expected);
}

#[test]
#[allow(clippy::too_many_lines)]
fn basic_claims_keep_generic_prior_play_and_pending_expected_identities() {
    let view = snapshot(VariantSpec::from_catalog(VariantId::Black6Suits));
    let hands = layouts(&view);
    let focus = hands[2][0];
    let hidden = hands[0][0];
    let expected = domain(&view.variant, 5, 3);
    let prior = HGroupClueInterpretation {
        turn: 0,
        giver: PlayerId::new(1),
        target: PlayerId::new(0),
        clue: GenericClue::Rank(hanabi_core::ClueRank::new(3)),
        touched: vec![hidden],
        stack_heights: [0; 5],
        focus: hidden,
        focus_was_chop: false,
        kind: HGroupClueKind::Play,
        focus_identities: expected,
        play_identities: expected,
        save_identities: IdentitySet::default(),
        new_non_focus: vec![],
        non_focus_identities: vec![],
        non_focus_trash_identities: vec![],
        previously_gotten: vec![],
        hypotheses: vec![],
        unresolved_visible_prefix: vec![],
    };
    let historical = HistoricalView::new(&view, 1);
    let facts = vec![ClueFacts::default(); 15];
    let gotten = [hidden].into_iter().collect();
    let mut convention = ConventionFacts::default();
    let mut pending = ConnectionManager::default();
    let check = |convention: &ConventionFacts,
                 clues: &[HGroupClueInterpretation],
                 pending: &ConnectionManager| {
        claimed_identities_at_clue(
            &view.variant,
            focus,
            &hands,
            &historical,
            &facts,
            convention,
            clues,
            &gotten,
            pending,
        )
    };
    assert_eq!(
        check(&convention, std::slice::from_ref(&prior), &pending),
        expected
    );
    let mut ambiguous = prior.clone();
    ambiguous.play_identities = expected.union(domain(&view.variant, 0, 3));
    assert!(check(&convention, &[ambiguous], &pending).is_empty());
    let mut saved = prior.clone();
    saved.play_identities = IdentitySet::default();
    saved.save_identities = expected;
    assert!(check(&convention, &[saved], &pending).is_empty());
    convention.set_fixed(&[hidden], true);
    assert!(check(&convention, &[prior], &pending).is_empty());
    let physical = Card::new(Suit::new(5), Rank::Three);
    pending.start(
        0,
        ConnectionObligation {
            promise: PromiseId::UNASSIGNED,
            actor: PlayerId::new(0),
            cards: vec![hidden],
            expected: physical,
            focus_identity: physical,
            kind: HGroupConnectionKind::Prompt,
            focus,
            step: 0,
        },
    );
    assert_eq!(check(&convention, &[], &pending), expected);
    // This checks claim storage/mapping only, not generic connection execution.
    assert!(
        claimed_identities_at_clue(
            &view.variant,
            hidden,
            &hands,
            &historical,
            &facts,
            &ConventionFacts::default(),
            &[],
            &gotten,
            &pending
        )
        .is_empty()
    );
    assert!(
        claimed_identities_at_clue(
            &view.variant,
            focus,
            &hands[1..],
            &historical,
            &facts,
            &ConventionFacts::default(),
            &[],
            &gotten,
            &pending
        )
        .is_empty()
    );
}
