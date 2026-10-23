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
// Temporary probe: deliberately fails at the first retained boundary.
#[test]
fn contract_v5_r2_legal_rainbow_clue_discard() {
    let variant = VariantSpec::from_catalog(VariantId::Rainbow6Suits);
    let deck = variant
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
    let mut state = FullState::new_variant(3, variant, deck).unwrap();
    let view = state.physical_view_for(PlayerId::new(0)).unwrap();
    let rank = view.hands[1][0].identity.unwrap().rank;
    state
        .apply_physical(hanabi_core::PhysicalAction::Clue {
            target: PlayerId::new(1),
            clue: GenericClue::Rank(ClueRank::new(rank.number())),
        })
        .unwrap();
    state.apply(Action::Discard(state.hands()[1][0])).unwrap();
    let view = state
        .physical_view_for(PlayerId::new(0))
        .unwrap()
        .try_standard()
        .unwrap();
    assert!(
        view.history
            .iter()
            .any(|entry| matches!(entry.event, ObservedEvent::Discarded { .. }))
    );
    let deductions = LogicalDeductions::new(view).unwrap();
    replay_h_group(&deductions, HGroupProfile::Level(HGroupLevel::Level1));
}

