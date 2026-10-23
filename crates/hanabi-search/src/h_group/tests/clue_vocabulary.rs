use super::*;
use hanabi_core::{ClueColorId, ClueRank, GenericClue, VariantId, VariantSpec};

#[test]
fn contract_replay_clue_vocabulary_retains_public_labels_and_input_parity() {
    // Existing replay provides real public labels; this test asserts only the
    // type/storage boundary and parity, not new convention interpretations.
    let state = expert_replay_p4v0s2().state_at_turn(8).unwrap();
    for observer in (0..4).map(PlayerId::new) {
        let standard = state.view_for(observer).unwrap();
        let physical = state.physical_view_for(observer).unwrap();
        let round_trip = physical.clone().try_standard().unwrap();
        let deductions = LogicalDeductions::new(standard).unwrap();
        let replay = replay_h_group(&deductions, HGroupProfile::Max);
        let equivalent = replay_h_group(
            &LogicalDeductions::new(round_trip).unwrap(),
            HGroupProfile::Max,
        );
        assert!(!replay.clues.is_empty());
        assert_eq!(replay.clues, equivalent.clues);
        for interpretation in &replay.clues {
            let entry = physical
                .history
                .iter()
                .find(|entry| {
                    entry.turn == interpretation.turn
                        && matches!(entry.event, ObservedEvent::Clued { .. })
                })
                .unwrap();
            let ObservedEvent::Clued { clue, touched, .. } = &entry.event else {
                unreachable!();
            };
            let stored: GenericClue = interpretation.clue;
            assert_eq!(stored, *clue);
            assert_eq!(&interpretation.touched, touched);
        }
    }
}

#[test]
fn contract_prior_clue_equality_uses_generic_labels_for_both_history_vocabularies() {
    let state = expert_replay_p4v0s2().state_at_turn(8).unwrap();
    let standard = state.view_for(PlayerId::new(0)).unwrap();
    let physical = state.physical_view_for(PlayerId::new(0)).unwrap();
    let labels = (0..5)
        .map(|color| GenericClue::Color(ClueColorId::new(color)))
        .chain((1..=6).map(|rank| GenericClue::Rank(ClueRank::new(rank))))
        .collect::<Vec<_>>();
    for entry in &physical.history {
        let ObservedEvent::Clued { touched, .. } = &entry.event else {
            continue;
        };
        for card in touched {
            // Test strict before-turn exclusion, positive label/card matches,
            // different labels, and a nonstandard label without legacy conversion.
            for turn in [entry.turn, entry.turn + 1] {
                for label in &labels {
                    let expected = physical.history.iter().any(|prior| {
                        prior.turn < turn
                            && matches!(&prior.event,
                            ObservedEvent::Clued { clue, touched, .. }
                            if clue == label && touched.contains(card))
                    });
                    assert_eq!(
                        was_clued_before_with(&physical, turn, *card, *label),
                        expected
                    );
                    assert_eq!(
                        was_clued_before_with(&standard, turn, *card, *label),
                        expected
                    );
                }
            }
        }
    }
    assert!(!was_clued_before_with(
        &physical,
        state.turn(),
        CardId::new(49),
        labels[0]
    ));
}

#[test]
#[should_panic(expected = "H-Group semantics require No Variant")]
fn standard_semantic_adapter_rejects_other_variants_even_with_standard_labels() {
    standard_h_group_clue(
        &VariantSpec::from_catalog(VariantId::Rainbow6Suits),
        GenericClue::from(Clue::Rank(Rank::One)),
    );
}

#[test]
#[should_panic(expected = "No Variant H-Group clue must be a standard label")]
fn standard_semantic_adapter_rejects_nonstandard_labels() {
    standard_h_group_clue(
        &VariantSpec::no_variant(),
        GenericClue::Rank(ClueRank::new(6)),
    );
}

#[test]
fn contract_physical_touch_and_literal_facts_preserve_every_standard_clue() {
    let state = expert_replay_p4v0s2().state_at_turn(8).unwrap();
    let view = state.view_for(PlayerId::new(0)).unwrap();
    for clue in Suit::ALL
        .into_iter()
        .map(Clue::Suit)
        .chain(Rank::ALL.into_iter().map(Clue::Rank))
    {
        let label = GenericClue::from(clue);
        assert_eq!(standard_h_group_clue(&view.variant, label), clue);
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                let card = Card::new(suit, rank);
                assert_eq!(
                    physical_clue_matches(&view, label, card),
                    clue.matches(card)
                );
            }
        }
        for positive in [false, true] {
            let mut standard = ClueFacts::default();
            if positive {
                standard.add_positive_clue(clue);
            } else {
                standard.add_negative_clue(clue);
            }
            let mut physical = ClueFacts::default();
            physical.record_physical(label, positive);
            assert_eq!(standard, physical);
        }
    }
}
// Physical/inference infrastructure contract, not a convention/optimal-move ruling.
#[test]
fn contract_v5_r2_legal_rainbow_clue_discard() {
    let variant = VariantSpec::from_catalog(VariantId::Rainbow6Suits);
    let deck: Vec<_> = variant
        .identities()
        .identities()
        .iter()
        .flat_map(|identity| {
            std::iter::repeat_n(
                Card::new(identity.suit, identity.rank),
                usize::from(identity.copies),
            )
        })
        .collect();
    let mut state = FullState::new_variant(3, variant.clone(), deck.clone()).unwrap();
    let mut reconstructed = FullState::new_variant(3, variant.clone(), deck).unwrap();
    let view = state.physical_view_for(PlayerId::new(0)).unwrap();
    let rank = view.hands[1][0].identity.unwrap().rank;
    let label = GenericClue::Rank(ClueRank::new(rank.number()));
    let action = hanabi_core::PhysicalAction::Clue {
        target: PlayerId::new(1),
        clue: label,
    };
    assert_eq!(state.clue_tokens(), 8);
    state.apply_physical(action).unwrap();
    reconstructed.apply_physical(action).unwrap();
    let before_discard = state.clone();
    let discarded = state.hands()[1][0];
    let revealed = view.hands[1][0].identity.unwrap();
    state.apply(Action::Discard(discarded)).unwrap();
    reconstructed
        .apply(Action::Discard(reconstructed.hands()[1][0]))
        .unwrap();
    let profile = HGroupProfile::Level(HGroupLevel::Level1);
    for observer in (0..3).map(PlayerId::new) {
        let physical = state.physical_view_for(observer).unwrap();
        assert_eq!(physical, reconstructed.physical_view_for(observer).unwrap());
        assert_eq!(physical.variant, variant);
        assert!(
            physical.hands[observer.index()]
                .iter()
                .all(|card| card.identity.is_none())
        );
        let view = physical.clone().try_standard().unwrap();
        assert_eq!(view.variant, variant);
        assert_eq!(view.history.len(), 3); // Clue, discard, draw; no filtered events.
        assert_eq!(view.clue_tokens, 8);
        let ObservedEvent::Clued {
            giver,
            target,
            clue,
            touched,
            untouched,
        } = &view.history[0].event
        else {
            panic!("missing clue")
        };
        assert_eq!(
            (*giver, *target, GenericClue::from(*clue)),
            (PlayerId::new(0), PlayerId::new(1), label)
        );
        assert!(touched.contains(&discarded));
        assert_eq!(touched.len() + untouched.len(), 5);
        assert!(
            matches!(view.history[1].event, ObservedEvent::Discarded { player, card, identity }
            if player == PlayerId::new(1) && card == discarded && identity == revealed)
        );
        assert!(
            matches!(view.history[2].event, ObservedEvent::Drew { player, identity, .. }
            if player == PlayerId::new(1) && identity.is_none() == (observer == player))
        );
        assert_eq!(view.history[0].turn, 0);
        assert_eq!(view.history[1].turn, 1);
        let prior = before_discard
            .physical_view_for(observer)
            .unwrap()
            .try_standard()
            .unwrap();
        assert_eq!(prior.history.len(), 1);
        assert_eq!(prior.clue_tokens, 7);
        assert!(prior.discard_pile.is_empty());
        assert_eq!(view.discard_pile, vec![(discarded, revealed)]);
        let before = replay_h_group(&LogicalDeductions::new(prior.clone()).unwrap(), profile);
        let deductions = LogicalDeductions::new(view.clone()).unwrap();
        let replay = replay_h_group(&deductions, profile);
        replay.validate().unwrap();
        assert_eq!(
            replay.hands,
            view.hands
                .iter()
                .map(|hand| hand.iter().map(|card| card.id).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        );
        assert_eq!(replay.clues.len(), 1);
        // Later public identity/replacement draw must not rewrite clue-time meaning.
        assert_eq!(replay.clues, before.clues);
        assert_eq!(replay.clues[0].clue, label);
        assert_eq!(&replay.clues[0].touched, touched);
        // The causal journal stores materialized changes, not no-op draw events.
        assert_eq!(replay.transitions.len(), 1);
        assert_eq!(
            replay
                .transitions
                .iter()
                .map(|event| event.turn)
                .collect::<Vec<_>>(),
            vec![0]
        );
        for card in touched {
            assert!(
                replay.transitions[0]
                    .delta
                    .card_changes
                    .iter()
                    .any(|change| change.card == *card
                        && change.fact
                            == super::super::transition::MaterializedCardFact::ExplicitlyClued
                        && change.kind == super::super::transition::FactChangeKind::Added)
            );
        }
        assert!(!replay.cards.already_playing.contains(&discarded));
        assert_eq!(
            super::super::identity::connection_heights(&view, Some(1)),
            vec![0; 6]
        );
        if observer == PlayerId::new(1) {
            assert_eq!(
                super::super::HistoricalView::new(&view, 1).identity(discarded),
                None
            );
            assert_eq!(
                super::super::HistoricalView::new(&view, 2).identity(discarded),
                Some(revealed)
            );
            let logical = LogicalDeductions::new(prior).unwrap();
            let domain = logical.possible_identities(discarded).unwrap();
            let sixth = variant
                .identities()
                .identities()
                .iter()
                .find(|identity| identity.suit == variant.stack_suits()[5] && identity.rank == rank)
                .unwrap();
            assert!(domain.contains_id(sixth.id));
            assert!(
                before.clues[0]
                    .non_focus_identities
                    .iter()
                    .any(|(_, identities)| identities.contains_id(sixth.id))
            );
            assert!(
                domain
                    .iter_in(variant.identities())
                    .all(|card| card.rank == rank)
            );
            // Literal rank evidence alone does not disclose a suit. Convention
            // focus may be narrower; its chronology is checked above by parity.
            assert!(domain.len() > 1);
        }
    }
}
