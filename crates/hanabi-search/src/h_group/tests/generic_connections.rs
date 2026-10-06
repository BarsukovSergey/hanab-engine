// Artificial snapshots exercise connection algorithm contracts, not optimal
// moves or new convention authorities. Reviewed No Variant replay tests remain
// the semantic baseline; these inputs vary the physical identity universe.
use super::*;
use crate::h_group::connection::ConnectionStatus;
use hanabi_core::{ClueColorId, ClueRank, IdentitySpec, VariantId, VariantSpec};

fn snapshot(variant: VariantSpec, players: u8) -> PlayerView {
    let mut view = FullState::new_standard(players, standard_deck())
        .unwrap()
        .view_for(PlayerId::new(0))
        .unwrap();
    view.play_stacks = vec![Vec::new(); variant.stack_suits().len()];
    view.variant = variant;
    view.history.clear();
    for card in view.hands.iter_mut().flatten() {
        card.identity = None;
    }
    view
}

fn facts(view: &PlayerView) -> Vec<ClueFacts> {
    let mut facts = vec![ClueFacts::default(); 64];
    for card in view.hands.iter().flatten() {
        facts[card.id.index()] = card.clues;
    }
    facts
}

fn layout(view: &PlayerView) -> Vec<Vec<CardId>> {
    view.hands
        .iter()
        .map(|hand| hand.iter().map(|card| card.id).collect())
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn plan(
    view: &PlayerView,
    giver: PlayerId,
    target: PlayerId,
    focus: CardId,
    identity: Card,
    clue: GenericClue,
    gotten: &CardSet,
    heights: &[usize],
) -> (Vec<ConnectionObligation>, ConnectionManager) {
    let mut pending = ConnectionManager::default();
    let mut invisible = ProvenancedCardSet::default();
    let facts = facts(view);
    let obligations = schedule_connection(
        HGroupProfile::Level(HGroupLevel::Level2),
        view,
        3,
        giver,
        target,
        focus,
        clue,
        &[focus],
        Some(identity),
        &layout(view),
        &facts,
        &facts,
        &[],
        gotten,
        &CardSet::default(),
        &CardSet::default(),
        &ConventionFacts::default(),
        &CardSet::default(),
        &mut invisible,
        heights,
        &mut pending,
        &mut None,
        false,
    );
    (obligations, pending)
}

#[test]
fn generic_connection_no_variant_and_sixth_suit_prompt_finesse_parity() {
    for (variant, suit) in [
        (VariantSpec::no_variant(), Suit::Red),
        (
            VariantSpec::from_catalog(VariantId::Rainbow6Suits),
            Suit::new(5),
        ),
    ] {
        for prompt in [false, true] {
            let mut view = snapshot(variant.clone(), 3);
            let focus = view.hands[2][0].id;
            let connector = view.hands[1].last().unwrap().id;
            let one = Card::new(suit, Rank::One);
            let two = Card::new(suit, Rank::Two);
            view.hands[1].last_mut().unwrap().identity = Some(one);
            view.hands[2][0]
                .clues
                .add_positive_clue(Clue::Rank(Rank::Two));
            let gotten = if prompt {
                view.hands[1]
                    .last_mut()
                    .unwrap()
                    .clues
                    .add_positive_clue(Clue::Rank(Rank::One));
                [connector].into_iter().collect()
            } else {
                CardSet::default()
            };
            let heights = vec![0; variant.stack_suits().len()];
            let (steps, pending) = plan(
                &view,
                PlayerId::new(0),
                PlayerId::new(2),
                focus,
                two,
                Clue::Rank(Rank::Two).into(),
                &gotten,
                &heights,
            );
            assert_eq!(steps.len(), 1);
            assert_eq!(steps[0].expected, one);
            assert_eq!(steps[0].cards, [connector]);
            assert_eq!(
                steps[0].kind,
                if prompt {
                    HGroupConnectionKind::Prompt
                } else {
                    HGroupConnectionKind::Finesse
                }
            );
            assert!(pending.validate().is_ok());
            assert_eq!(
                snapshot_play_identities(
                    HGroupProfile::Level(HGroupLevel::Level2),
                    connection_domain(&view, two),
                    PlayerId::new(0),
                    PlayerId::new(2),
                    focus,
                    &view,
                    &layout(&view),
                    &facts(&view),
                    &gotten,
                    &CardSet::default(),
                    &ConnectionManager::default(),
                    &ConventionFacts::default(),
                    &CardSet::default(),
                    &heights,
                    3,
                    false
                ),
                connection_domain(&view, two)
            );
            assert!(
                interpretation::delayed_connection_score(
                    &view,
                    HGroupProfile::Level(HGroupLevel::Level2),
                    PlayerId::new(2),
                    focus,
                    two,
                    false,
                    &CardSet::default(),
                    &gotten,
                    &CardSet::default(),
                    &ConnectionManager::default(),
                    &ConventionFacts::default(),
                    &CardSet::default(),
                    &[]
                )
                .is_some()
            );
        }
    }
}

#[test]
fn generic_connection_prompt_precedes_earlier_finesse_in_six_player_order() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 6);
    let one = Card::new(Suit::new(5), Rank::One);
    let two = Card::new(Suit::new(5), Rank::Two);
    let focus = view.hands[5][0].id;
    view.hands[1].last_mut().unwrap().identity = Some(one);
    let prompt = view.hands[4].last_mut().unwrap();
    prompt.identity = Some(one);
    prompt.clues.add_positive_clue(Clue::Rank(Rank::One));
    let prompted = prompt.id;
    let gotten = [prompted].into_iter().collect();
    let (steps, _) = plan(
        &view,
        PlayerId::new(0),
        PlayerId::new(5),
        focus,
        two,
        Clue::Rank(Rank::Two).into(),
        &gotten,
        &[0; 6],
    );
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].actor, PlayerId::new(4));
    assert_eq!(steps[0].cards, [prompted]);
    assert_eq!(steps[0].kind, HGroupConnectionKind::Prompt);
}

#[test]
fn generic_connection_reverse_finesse_uses_non_legacy_identity() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 4);
    let focus = view.hands[1][0].id;
    view.hands[1].truncate(1);
    view.hands[2].clear();
    let connector = view.hands[3].last_mut().unwrap();
    connector.identity = Some(Card::new(Suit::new(5), Rank::One));
    let card = connector.id;
    let two = Card::new(Suit::new(5), Rank::Two);
    let (steps, _) = plan(
        &view,
        PlayerId::new(0),
        PlayerId::new(1),
        focus,
        two,
        Clue::Rank(Rank::Two).into(),
        &CardSet::default(),
        &[0; 6],
    );
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].actor, PlayerId::new(3));
    assert_eq!(steps[0].cards, [card]);
    assert_eq!(
        snapshot_play_identities(
            HGroupProfile::Level(HGroupLevel::Level2),
            connection_domain(&view, two),
            PlayerId::new(0),
            PlayerId::new(1),
            focus,
            &view,
            &layout(&view),
            &facts(&view),
            &CardSet::default(),
            &CardSet::default(),
            &ConnectionManager::default(),
            &ConventionFacts::default(),
            &CardSet::default(),
            &[0; 6],
            3,
            false
        ),
        connection_domain(&view, two)
    );
}

#[test]
fn generic_connection_self_finesse_requires_no_direct_alternative_and_ignores_hidden_truth() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 3);
    let focus = view.hands[0][0].id;
    view.hands[1].clear();
    view.hands[2].clear();
    let one = connection_domain(&view, Card::new(Suit::new(5), Rank::One));
    let two = connection_domain(&view, Card::new(Suit::new(5), Rank::Two));
    for hidden in [
        None,
        Some(Card::new(Suit::Red, Rank::Five)),
        Some(Card::new(Suit::new(5), Rank::One)),
    ] {
        view.hands[0].last_mut().unwrap().identity = hidden;
        for direct_alternative in [false, true] {
            let identities = if direct_alternative {
                one.union(two)
            } else {
                two
            };
            let result = snapshot_play_identities(
                HGroupProfile::Level(HGroupLevel::Level2),
                identities,
                PlayerId::new(2),
                PlayerId::new(0),
                focus,
                &view,
                &layout(&view),
                &facts(&view),
                &CardSet::default(),
                &CardSet::default(),
                &ConnectionManager::default(),
                &ConventionFacts::default(),
                &CardSet::default(),
                &[0; 6],
                3,
                false,
            );
            assert_eq!(result, if direct_alternative { one } else { two });
            let (steps, _) = plan(
                &view,
                PlayerId::new(2),
                PlayerId::new(0),
                focus,
                Card::new(Suit::new(5), Rank::Two),
                if direct_alternative {
                    Clue::Suit(Suit::Red).into()
                } else {
                    Clue::Rank(Rank::Two).into()
                },
                &CardSet::default(),
                &[0; 6],
            );
            assert_eq!(steps.is_empty(), direct_alternative);
            if let Some(step) = steps.first() {
                assert_eq!(step.actor, PlayerId::new(0));
                assert_eq!(step.kind, HGroupConnectionKind::Finesse);
            }
        }
    }
}

fn irregular_variant() -> VariantSpec {
    // Universe and stack order deliberately differ from Card::index(), and
    // physical ranks are nonconsecutive. No convention pack is introduced.
    let suits = vec![Suit::new(5), Suit::Red];
    let ranks = vec![Rank::new(1), Rank::new(3), Rank::new(5)];
    let identities = suits
        .iter()
        .rev()
        .flat_map(|suit| {
            ranks.iter().rev().map(move |rank| IdentitySpec {
                suit: *suit,
                rank: *rank,
                copies: 4,
                clue_colors: vec![ClueColorId::new(0)],
                clue_ranks: vec![ClueRank::new(rank.number())],
            })
        })
        .collect();
    VariantSpec::new(
        "Connection contract",
        suits,
        ranks,
        vec![ClueColorId::new(0)],
        vec![ClueRank::new(1), ClueRank::new(3), ClueRank::new(5)],
        identities,
    )
    .unwrap()
}

#[test]
fn generic_connection_multi_step_rank_order_preserves_lifecycle_and_provenance() {
    let mut view = snapshot(irregular_variant(), 4);
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    let five = Card::new(Suit::new(5), Rank::Five);
    let first = view.hands[1].last().unwrap().id;
    let second = view.hands[2].last().unwrap().id;
    let focus = view.hands[3][0].id;
    view.hands[1].last_mut().unwrap().identity = Some(one);
    view.hands[2].last_mut().unwrap().identity = Some(three);
    view.hands[3][0]
        .clues
        .add_positive_clue(Clue::Rank(Rank::Five));
    let (steps, mut pending) = plan(
        &view,
        PlayerId::new(0),
        PlayerId::new(3),
        focus,
        five,
        Clue::Rank(Rank::Five).into(),
        &CardSet::default(),
        &[0; 2],
    );
    assert_eq!(
        steps.iter().map(|step| step.expected).collect::<Vec<_>>(),
        [one, three]
    );
    assert_eq!(
        steps.iter().map(|step| step.step).collect::<Vec<_>>(),
        [0, 1]
    );
    let provenance = pending.provenance(steps[0].promise).unwrap();
    assert_eq!(provenance.expected, one);
    assert_eq!(provenance.created_turn, 3);
    assert!(!pending.is_active(&steps[1]));
    let clue = GenericClue::Color(ClueColorId::new(0));
    assert_eq!(
        pending.match_clue(&view.variant, PlayerId::new(2), clue, &[second]),
        ConnectionClueMatch::LaterContinuation
    );
    let mut heights = vec![0; 2];
    assert!(connection_playable(&view, &heights, one));
    pending.advance_play(4, PlayerId::new(1), first, one, true);
    connection_advance(&view, &mut heights, one);
    assert!(pending.is_active(&steps[1]));
    assert_eq!(
        pending.match_clue(&view.variant, PlayerId::new(2), clue, &[second]),
        ConnectionClueMatch::ActiveRedundant
    );
    assert!(connection_playable(&view, &heights, three));
    pending.advance_play(5, PlayerId::new(2), second, three, true);
    connection_advance(&view, &mut heights, three);
    assert!(connection_playable(&view, &heights, five));
    assert!(pending.is_empty());
    assert_eq!(pending.provenance(steps[0].promise), Some(provenance));
    assert_eq!(
        pending
            .transitions()
            .iter()
            .filter(|transition| transition.to == ConnectionStatus::Satisfied)
            .count(),
        2
    );
    assert!(pending.validate().is_ok());
}

#[test]
fn generic_connection_clue_matching_uses_rainbow_and_pink_touches() {
    for variant_id in [VariantId::Rainbow6Suits, VariantId::Pink6Suits] {
        let mut view = snapshot(VariantSpec::from_catalog(variant_id), 3);
        let focus = view.hands[2][0].id;
        let connector = view.hands[1].last_mut().unwrap();
        connector.identity = Some(Card::new(Suit::new(5), Rank::One));
        let card = connector.id;
        let (_, pending) = plan(
            &view,
            PlayerId::new(0),
            PlayerId::new(2),
            focus,
            Card::new(Suit::new(5), Rank::Two),
            Clue::Rank(Rank::Two).into(),
            &CardSet::default(),
            &[0; 6],
        );
        let clue = match variant_id {
            VariantId::Rainbow6Suits => Clue::Suit(Suit::Red).into(),
            VariantId::Pink6Suits => Clue::Rank(Rank::Five).into(),
            _ => unreachable!(),
        };
        assert_eq!(
            pending.match_clue(&view.variant, PlayerId::new(1), clue, &[card]),
            ConnectionClueMatch::ActiveRedundant
        );
        assert_eq!(
            pending.match_clue(&VariantSpec::no_variant(), PlayerId::new(1), clue, &[card]),
            ConnectionClueMatch::Unrelated
        );
    }
}

#[test]
fn generic_connection_loaded_sixth_suit_progresses_through_two_predecessors() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 4);
    let focus = view.hands[3][0].id;
    let one = Card::new(Suit::new(5), Rank::One);
    let two = Card::new(Suit::new(5), Rank::Two);
    let three = Card::new(Suit::new(5), Rank::Three);
    view.hands[1].last_mut().unwrap().identity = Some(one);
    view.hands[2].last_mut().unwrap().identity = Some(two);
    let mut pending = ConnectionManager::default();
    pending.start(
        1,
        ConnectionObligation {
            promise: PromiseId::UNASSIGNED,
            actor: PlayerId::new(3),
            cards: vec![view.hands[3][1].id],
            expected: Card::new(Suit::Red, Rank::One),
            focus_identity: Card::new(Suit::Red, Rank::Two),
            kind: HGroupConnectionKind::Finesse,
            focus: view.hands[3][2].id,
            step: 0,
        },
    );
    let loaded = |view: &PlayerView, heights: &[usize]| {
        interpretation::loaded_connection_plan(
            view,
            Some(&layout(view)),
            Some(&facts(view)),
            None,
            PlayerId::new(0),
            PlayerId::new(3),
            focus,
            three,
            &CardSet::default(),
            &CardSet::default(),
            &pending,
            heights,
        )
    };
    assert_eq!(loaded(&view, &[0; 6]), Some(None));
    view.hands[2].last_mut().unwrap().identity = Some(Card::new(Suit::Blue, Rank::Five));
    // The first available predecessor must not fabricate the second one.
    assert_eq!(loaded(&view, &[0; 6]), None);
    let mut heights = vec![0; 6];
    connection_advance(&view, &mut heights, one);
    view.hands[2].last_mut().unwrap().identity = Some(two);
    assert_eq!(loaded(&view, &heights), Some(None));
}

#[test]
fn generic_connection_deferred_prompt_and_fix_keep_promise_history() {
    let mut view = snapshot(irregular_variant(), 3);
    let focus = view.hands[2][0].id;
    let expected = Card::new(Suit::new(5), Rank::Three);
    let focus_identity = Card::new(Suit::new(5), Rank::Five);
    view.hands[1].last_mut().unwrap().identity = Some(expected);
    let (steps, mut pending) = plan(
        &view,
        PlayerId::new(0),
        PlayerId::new(2),
        focus,
        focus_identity,
        Clue::Rank(Rank::Five).into(),
        &CardSet::default(),
        &[1, 0],
    );
    assert_eq!(steps.len(), 1);
    let original = pending.provenance(steps[0].promise).unwrap();
    let fallback = ConnectionObligation {
        actor: PlayerId::new(2),
        cards: vec![view.hands[2][1].id],
        kind: HGroupConnectionKind::Prompt,
        ..steps[0].clone()
    };
    pending.defer_connection(steps[0].promise, fallback);
    pending.resolve_deferred_connections(
        &view.variant,
        4,
        PlayerId::new(1),
        true,
        false,
        &[0; 2],
        &[0; 2],
    );
    assert_eq!(pending.iter().next().unwrap().promise, original.id);
    pending.resolve_deferred_connections(
        &view.variant,
        5,
        PlayerId::new(1),
        true,
        false,
        &[1, 0],
        &[1, 0],
    );
    let active = pending.iter().next().unwrap().clone();
    assert_eq!(active.kind, HGroupConnectionKind::Prompt);
    assert_ne!(active.promise, original.id);
    assert_eq!(pending.provenance(original.id), Some(original));
    pending.cancel_where(6, ConnectionTransitionReason::Fixed, |step| {
        step.promise == active.promise
    });
    assert!(pending.is_empty());
    assert_eq!(
        pending.transitions().last().unwrap().to,
        ConnectionStatus::AwaitingFix
    );
    assert!(pending.provenance(active.promise).is_some());
}

#[test]
fn generic_connection_historical_dynamic_stacks_exclude_future_and_failed_plays() {
    let mut view = snapshot(irregular_variant(), 3);
    for (turn, rank, successful) in [
        (4, Rank::One, true),
        (7, Rank::Three, true),
        (8, Rank::Five, false),
    ] {
        view.history.push(ObservedHistoryEntry {
            turn,
            event: ObservedEvent::Played {
                player: PlayerId::new(1),
                card: CardId::new(30 + usize::try_from(turn).unwrap()),
                identity: Card::new(Suit::new(5), rank),
                successful,
            },
        });
    }
    assert_eq!(identity::connection_heights(&view, Some(4)), [0, 0]);
    assert_eq!(identity::connection_heights(&view, Some(7)), [1, 0]);
    assert_eq!(identity::connection_heights(&view, Some(8)), [2, 0]);
    assert_eq!(identity::connection_heights(&view, Some(9)), [2, 0]);
}

#[test]
fn generic_connection_historical_speculation_and_commit_share_the_active_plan() {
    let mut view = snapshot(irregular_variant(), 4);
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    let five = Card::new(Suit::new(5), Rank::Five);
    view.hands[1].last_mut().unwrap().identity = Some(one);
    view.hands[2].last_mut().unwrap().identity = Some(three);
    let focus = view.hands[3][0].id;
    let hands = layout(&view);
    let facts = facts(&view);
    let empty = CardSet::default();
    let convention_facts = ConventionFacts::default();
    let context = ConnectionPlanningContext {
        profile: HGroupProfile::Level(HGroupLevel::Level2),
        view: &view,
        turn: 3,
        giver: PlayerId::new(0),
        target: PlayerId::new(3),
        focus,
        clue: Clue::Rank(Rank::Five).into(),
        touches: CurrentClueTouches(&[focus]),
        hands: &hands,
        facts: &facts,
        facts_before: &facts,
        clues: &[],
        promptable_before: PromptableBeforeClue(&empty),
        protected_before: &empty,
        already_playing: &empty,
        declined_direct_plays: &empty,
        convention_facts: &convention_facts,
        chop_moved: &empty,
        stack_heights: &[0; 2],
        allow_blind_reverse_empathy: false,
    };
    let mut pending = ConnectionManager::default();
    let mut invisible = ProvenancedCardSet::default();
    let hypothesis = context.simulate(five, &pending, &invisible);
    assert!(pending.is_empty());
    assert_eq!(
        hypothesis
            .connection_steps
            .iter()
            .map(|step| step.expected)
            .collect::<Vec<_>>(),
        [one, three]
    );
    let (committed, fix) = context.commit(Some(five), &mut pending, &mut invisible);
    assert!(fix.is_none());
    assert_eq!(committed.len(), hypothesis.connection_steps.len());
    for (step, promised) in committed.iter().zip(&hypothesis.connection_steps) {
        assert_eq!(
            (step.actor, &step.cards, step.expected, step.kind),
            (
                promised.actor,
                &promised.cards,
                promised.expected,
                promised.kind
            )
        );
        assert_eq!(pending.provenance(step.promise).unwrap().focus, focus);
    }
    assert!(pending.validate().is_ok());
}

#[test]
#[allow(clippy::too_many_lines)]
fn generic_connection_owner_notes_and_turn_projection_use_active_identity_ids() {
    let variant = irregular_variant();
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    let five = Card::new(Suit::new(5), Rank::Five);
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
    let position = deck.iter().position(|card| *card == one).unwrap();
    deck.swap(0, position);
    let view = FullState::new_variant(3, variant, deck)
        .unwrap()
        .physical_view_for(PlayerId::new(1))
        .unwrap()
        // This empty-history adapter retains VariantSpec for the connection
        // kernel; the No Variant production-policy gate remains in place.
        .try_standard()
        .unwrap();
    let deductions = LogicalDeductions::new(view.clone()).unwrap();
    let first = view.hands[1][0].id;
    let later = view.hands[1][1].id;
    let focus = view.hands[2][0].id;
    let mut replay = HGroupState {
        strategic_deductions: vec![],
        hands: layout(&view),
        cards: ConventionCardState {
            explicitly_clued: Default::default(),
            invisibly_clued: Default::default(),
            already_playing: Default::default(),
            chop_moved: Default::default(),
            discard_now: vec![],
            forced_playable: Default::default(),
            invalidated_focuses: Default::default(),
            declined_direct_plays: Default::default(),
            facts: Default::default(),
        },
        clues: vec![],
        pending_connections: Default::default(),
        early_game: true,
        signals: Default::default(),
        must_clue: Default::default(),
        implicit_saves: vec![],
        required_fixes: Default::default(),
        transitions: vec![],
        knowledge: Default::default(),
    };
    for (step, card, expected) in [(0, first, one), (1, later, three)] {
        replay.pending_connections.start(
            0,
            ConnectionObligation {
                promise: PromiseId::UNASSIGNED,
                actor: view.observer,
                cards: vec![card],
                expected,
                focus_identity: five,
                kind: HGroupConnectionKind::Finesse,
                focus,
                step,
            },
        );
    }
    let knowledge = build_convention_knowledge(&deductions, &replay);
    let notes = knowledge.project(&deductions);
    let active = notes.iter().find(|note| note.card == first).unwrap();
    let queued = notes.iter().find(|note| note.card == later).unwrap();
    assert_eq!(active.identities, connection_domain(&view, one));
    assert_eq!(active.promised_identity, Some(one));
    assert_eq!(
        active.play_obligation,
        Some(HGroupPlayObligation::Connection(
            HGroupConnectionKind::Finesse
        ))
    );
    assert_eq!(queued.identities, connection_domain(&view, three));
    assert_eq!(queued.promised_identity, Some(three));
    assert!(queued.play_obligation.is_none());
    assert!(queued.finessed);
    assert!(model::connection_candidate_is_eligible(
        HGroupConnectionKind::Prompt,
        three,
        later,
        &[first, later],
        &notes,
        &deductions
    ));
    assert!(!model::connection_candidate_is_eligible(
        HGroupConnectionKind::Prompt,
        three,
        first,
        &[first, later],
        &notes,
        &deductions
    ));

    // Project the same physical stack through a preceding player's due step.
    replay.pending_connections = ConnectionManager::default();
    replay.pending_connections.start(
        0,
        ConnectionObligation {
            promise: PromiseId::UNASSIGNED,
            actor: PlayerId::new(0),
            cards: vec![view.hands[0][0].id],
            expected: one,
            focus_identity: five,
            kind: HGroupConnectionKind::Finesse,
            focus,
            step: 0,
        },
    );
    let schedule = ActionSchedule::from_replay(&view, &replay);
    assert_eq!(
        schedule.stack_heights_before_in(&view, view.observer),
        [1, 0]
    );
}
