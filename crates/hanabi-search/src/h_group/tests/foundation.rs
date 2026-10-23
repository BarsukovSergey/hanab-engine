//! Source-backed foundation bug reproductions, not self-play strategy oracles.
use super::*;

fn white_seed_one() -> HanabiLiveReplay {
    // Source: pilot SHA 5f7585da1da6a2b45d6a6551c84eafea304a834f,
    // white-hgroup-5f7585d.json SHA256
    // 7272f774e6e88c6fb2c809ec33e44e1ebdf6b4cfea6187d5492855460ea9079a.
    // Exact ordered deck and first 50 recorded actions; no diagnostic meanings.
    HanabiLiveReplay::from_json(include_str!("fixtures/white-seed1-foundation.json"))
        .expect("recovered pilot source data parses")
}

#[test]
fn contract_foundation_white_recorded_prefix_is_legal_and_redacted() {
    let fixture = white_seed_one();
    assert_eq!(fixture.actions.len(), 50);
    let after_strike = fixture.replay().expect("all 50 recorded actions are legal");
    assert_eq!(after_strike.turn(), 50);
    assert_eq!(after_strike.strikes(), 1);
    let state = fixture.state_at_turn(45).unwrap();
    assert_eq!(
        state.card(CardId::new(39)),
        Some(Card::new(Suit::Red, Rank::Three))
    );
    assert_eq!(
        state.card(CardId::new(40)),
        Some(Card::new(Suit::new(5), Rank::Two))
    );
    let alice = state
        .physical_view_for(PlayerId::new(0))
        .unwrap()
        .try_standard()
        .unwrap();
    assert_eq!(identity_of(&alice, CardId::new(39)), None);
    assert_eq!(
        HistoricalView::new(&alice, 44).identity(CardId::new(40)),
        Some(Card::new(Suit::new(5), Rank::Two))
    );
    // Bob's later strike reveals White2 publicly, but it was hidden to him at
    // the Red clue. Alice could already see it then. Rebuilding either view
    // after the public reveal must preserve this historical distinction.
    for observer in [PlayerId::new(0), PlayerId::new(1)] {
        let later = after_strike
            .physical_view_for(observer)
            .unwrap()
            .try_standard()
            .unwrap();
        assert_eq!(
            identity_of(&later, CardId::new(40)),
            Some(Card::new(Suit::new(5), Rank::Two))
        );
        assert_eq!(
            HistoricalView::new(&later, 44).identity(CardId::new(40)),
            if observer == PlayerId::new(0) {
                Some(Card::new(Suit::new(5), Rank::Two))
            } else {
                None
            }
        );
    }
}

#[test]
fn contract_foundation_white_finesse_hypotheses_respect_observer_visible_cards() {
    let fixture = white_seed_one();
    let state = fixture.state_at_turn(45).unwrap();
    // Ordinary software invariant, not a ruling that this clue is good or that
    // Alice must play. Official L1 Finesse/Empathy and L3 Information Lock:
    // an observer's hidden card may be hypothesized, but a different player's
    // visibly White2 cannot become Red3. Inspect at the clue's event time.
    // https://hanabi.github.io/beginner/finesse/
    // https://hanabi.github.io/level-3/#information-lock-principle
    for observer in [PlayerId::new(0), PlayerId::new(1), PlayerId::new(2)] {
        let view = state
            .physical_view_for(observer)
            .unwrap()
            .try_standard()
            .unwrap();
        let deductions = LogicalDeductions::new(view.clone()).unwrap();
        let replay = replay_h_group(&deductions, HGroupProfile::Level(HGroupLevel::Level4));
        let clue = replay
            .clues
            .iter()
            .find(|clue| clue.turn == 44)
            .expect("recorded Red clue retained");
        assert_eq!(clue.focus, CardId::new(32));
        for hypothesis in &clue.hypotheses {
            for step in &hypothesis.connection_steps {
                for &card in &step.cards {
                    if let Some(visible) = HistoricalView::new(&view, clue.turn).identity(card) {
                        assert_eq!(
                            visible, step.expected,
                            "observer {observer:?}, focus {:?}, step {step:?}",
                            hypothesis.focus_identity
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn contract_foundation_white_finesse_response_reaches_executor_and_recipient() {
    let fixture = white_seed_one();
    let profile = HGroupProfile::Level(HGroupLevel::Level4);
    let mut state = fixture.state_at_turn(45).unwrap();
    let view = state
        .physical_view_for(PlayerId::new(0))
        .unwrap()
        .try_standard()
        .unwrap();
    // Official beginner Finesse example: a non-chop Red4 above Red2, with
    // no visible external Red3 connector, promises the blind actor's Red3.
    // This assertion follows that conditional reading, not the pilot discard.
    // https://hanabi.github.io/beginner/finesse/
    assert_eq!(state.play_stacks()[Suit::Red.index()].len(), 2);
    assert_eq!(view.hands[0].last().unwrap().id, CardId::new(39));
    let facts = view.hands[0].last().unwrap().clues;
    assert!(
        Suit::ALL
            .iter()
            .all(|&suit| !facts.has_positive_clue(Clue::Suit(suit)))
    );
    assert!(
        Rank::ALL
            .iter()
            .all(|&rank| !facts.has_positive_clue(Clue::Rank(rank)))
    );
    let deductions = LogicalDeductions::new(view).unwrap();
    let analysis = crate::SupportedConvention::HGroup(profile).analyze(&deductions);
    assert_eq!(analysis.forced_action, Some(Action::Play(CardId::new(39))));
    assert!(
        analysis
            .actions
            .iter()
            .any(|candidate| candidate.action == Action::Play(CardId::new(39)))
    );

    // Hypothetical branch from the recorded position, not a fixture rewrite:
    // demonstrating Red3 lets Bob resolve and play the promised Red4.
    state
        .apply_physical(hanabi_core::PhysicalAction::Play(CardId::new(39)))
        .unwrap();
    let bob = state
        .physical_view_for(PlayerId::new(1))
        .unwrap()
        .try_standard()
        .unwrap();
    let deductions = LogicalDeductions::new(bob).unwrap();
    let inferred = infer_h_group(&deductions, profile);
    assert!(inferred.playable_now.contains(&CardId::new(32)));
    assert!(!inferred.playable_now.contains(&CardId::new(40)));
}

#[test]
#[ignore = "manual admission-stage evidence; not a convention expectation"]
fn foundation_white_recorded_clue_admission_trace() {
    let fixture = white_seed_one();
    let profile = HGroupProfile::Level(HGroupLevel::Level4);
    let state = fixture.state_at_turn(44).unwrap();
    let view = state
        .physical_view_for(state.current_player())
        .unwrap()
        .try_standard()
        .unwrap();
    let deductions = LogicalDeductions::new(view).unwrap();
    let replay = replay_h_group(&deductions, profile);
    let action = Action::Clue {
        target: PlayerId::new(1),
        clue: Clue::Suit(Suit::Red),
    };
    let proposed = super::super::interpretation::h_group_clue_candidates_from_replay_inner(
        &deductions,
        profile,
        &replay,
    );
    for proposal in proposed.iter().filter(|proposal| proposal.action == action) {
        eprintln!(
            "P0 proposal: purpose={:?}, move={:?}, required_response={:?}",
            proposal.purpose(),
            proposal.move_kind(),
            proposal.required_response
        );
    }
    let _ = crate::capture_decisions(|| {
        let compiled = super::super::candidate_pipeline::compile(&deductions, profile, &replay);
        let meaning = crate::diagnostics::meaning(deductions.view(), action);
        for candidate in compiled
            .admitted
            .iter()
            .filter(|candidate| candidate.action == action)
        {
            let explanation = candidate.explanation();
            eprintln!(
                "P0 admitted: recognition={}, recipient_kind={:?}, required_response={:?}, checks={:?}",
                explanation.recognition,
                meaning.as_ref().map(|clue| clue.kind),
                candidate.required_response,
                explanation.validation
            );
        }
        for rejection in compiled
            .rejected
            .iter()
            .filter(|rejection| rejection.action == action)
        {
            eprintln!("P0 rejected: {rejection:?}");
        }
    });
    let after = fixture.state_at_turn(45).unwrap();
    for observer in [PlayerId::new(0), PlayerId::new(1)] {
        let view = after
            .physical_view_for(observer)
            .unwrap()
            .try_standard()
            .unwrap();
        let deductions = LogicalDeductions::new(view).unwrap();
        let inferred = infer_h_group(&deductions, profile);
        eprintln!(
            "P0 after clue, observer {observer:?}: connection={:?}, due={:?}",
            inferred.connection,
            super::super::decision::preferred_due_play_card(deductions.view(), &inferred, profile)
        );
    }
}
