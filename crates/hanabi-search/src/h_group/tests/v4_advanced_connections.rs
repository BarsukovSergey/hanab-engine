//! Artificial inputs test physical/epistemic algorithm contracts, not new
//! convention rulings or optimal moves. The six requested reviewed replay
//! regressions remain the No Variant semantic authorities.
use super::*;
use crate::h_group::identity::{
    connection_advance, connection_bit, connection_domain, connection_heights, connection_playable,
    connection_predecessors,
};
use crate::h_group::recognition::{
    apply_bluff_effects, apply_double_bluff_effects, apply_ejection_discharge_effects,
    apply_intermediate_bluff_effects, apply_priority_effects, retract_loaded_priority,
};
use hanabi_core::{ClueColorId, ClueRank, IdentitySpec, VariantId, VariantSpec};

fn snapshot(variant: VariantSpec, players: u8) -> PlayerView {
    let mut view = FullState::new_standard(players, standard_deck())
        .unwrap()
        .view_for(PlayerId::new(0))
        .unwrap();
    view.play_stacks = vec![vec![]; variant.stack_suits().len()];
    view.variant = variant;
    view.turn = 4;
    view.history.clear();
    for card in view.hands.iter_mut().flatten() {
        card.identity = None;
    }
    view
}

fn reordered() -> VariantSpec {
    let suits = vec![Suit::new(5), Suit::Red];
    let ranks = vec![Rank::One, Rank::Three, Rank::Five];
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
        "V4 physical contract",
        suits,
        ranks,
        vec![ClueColorId::new(0)],
        vec![ClueRank::new(1), ClueRank::new(3), ClueRank::new(5)],
        identities,
    )
    .unwrap()
}

fn layout(view: &PlayerView) -> Vec<Vec<CardId>> {
    view.hands
        .iter()
        .map(|hand| hand.iter().map(|card| card.id).collect())
        .collect()
}
fn facts(view: &PlayerView) -> Vec<ClueFacts> {
    let mut result = vec![ClueFacts::default(); 64];
    for card in view.hands.iter().flatten() {
        result[card.id.index()] = card.clues;
    }
    result
}
fn clue_entry(target: u8, clue: Clue, focus: CardId) -> ObservedHistoryEntry {
    ObservedHistoryEntry {
        turn: 4,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(0),
            target: PlayerId::new(target),
            clue,
            touched: vec![focus],
            untouched: vec![],
        },
    }
}
fn context<'a>(view: &'a PlayerView, entry: &'a ObservedHistoryEntry) -> HGroupTurnContext<'a> {
    let heights = connection_heights(view, None);
    HGroupTurnContext {
        entry,
        historical: HistoricalView::new(view, entry.turn),
        before: HGroupTurnSnapshot {
            hands: layout(view),
            facts: facts(view),
            stack_heights: [0; 5],
            dynamic_stack_heights: heights.clone(),
            clue_tokens: 7,
            deck_size: 30,
            early_game: false,
            protected: CardSet::default(),
            already_playing: CardSet::default(),
            forced_playable: CardSet::default(),
            older_play_obligations: CardSet::default(),
        },
        after: HGroupTurnView {
            hands: &[],
            facts: &[],
            stack_heights: [0; 5],
            dynamic_stack_heights: heights,
            clue_tokens: 7,
            deck_size: 30,
            early_game: false,
        },
        actor_before: ActorBeliefBefore::default(),
    }
}
fn interpretation(
    view: &PlayerView,
    focus: CardId,
    identity: Card,
    target: u8,
    clue: Clue,
) -> HGroupClueInterpretation {
    let domain = connection_domain(view, identity);
    HGroupClueInterpretation {
        turn: 4,
        giver: PlayerId::new(0),
        target: PlayerId::new(target),
        clue: clue.into(),
        touched: vec![focus],
        stack_heights: [0; 5],
        focus,
        focus_was_chop: false,
        kind: HGroupClueKind::Play,
        focus_identities: domain,
        play_identities: domain,
        save_identities: IdentitySet::default(),
        new_non_focus: vec![],
        non_focus_identities: vec![],
        non_focus_trash_identities: vec![],
        previously_gotten: vec![],
        hypotheses: vec![],
        unresolved_visible_prefix: vec![],
    }
}
#[derive(Default)]
struct Effects {
    explicit: ProvenancedCardSet,
    invisible: ProvenancedCardSet,
    playing: ProvenancedCardSet,
    pending: ConnectionManager,
    moved: ProvenancedCardSet,
    must_clue: PlayerSet,
    forced: ProvenancedCardSet,
    discard: Vec<CardId>,
    saves: Vec<(CardId, IdentitySet)>,
    fixes: FixObligations,
    signals: ConventionJournal,
}
impl Effects {
    fn borrow<'a>(&'a mut self, clues: &'a [HGroupClueInterpretation]) -> HGroupRuleEffects<'a> {
        HGroupRuleEffects {
            explicitly_clued: &self.explicit,
            invisibly_clued: &mut self.invisible,
            clues,
            already_playing: &mut self.playing,
            pending: &mut self.pending,
            chop_moved: &mut self.moved,
            must_clue: &mut self.must_clue,
            forced_playable: &mut self.forced,
            discard_now: &mut self.discard,
            implicit_saves: &mut self.saves,
            required_fixes: &mut self.fixes,
            signals: &mut self.signals,
        }
    }
}
fn obligation(
    actor: u8,
    cards: Vec<CardId>,
    expected: Card,
    focus: CardId,
    focus_identity: Card,
    kind: HGroupConnectionKind,
) -> ConnectionObligation {
    ConnectionObligation {
        promise: PromiseId::UNASSIGNED,
        actor: PlayerId::new(actor),
        cards,
        expected,
        focus_identity,
        kind,
        focus,
        step: 0,
    }
}

#[test]
fn contract_basic_bluff_active_predecessor_and_no_variant_precedence_parity() {
    for (variant, suit) in [
        (VariantSpec::no_variant(), Suit::Red),
        (
            VariantSpec::from_catalog(VariantId::Rainbow6Suits),
            Suit::new(5),
        ),
    ] {
        for older in [
            None,
            Some(HGroupConnectionKind::Finesse),
            Some(HGroupConnectionKind::Prompt),
        ] {
            let mut view = snapshot(variant.clone(), 3);
            let focus = view.hands[2][0].id;
            let blind = view.hands[1].last().unwrap().id;
            let two = Card::new(suit, Rank::Two);
            view.hands[2][0].identity = Some(two);
            view.hands[1].last_mut().unwrap().identity = Some(Card::new(Suit::Green, Rank::One));
            let entry = clue_entry(2, Clue::Suit(Suit::Red), focus);
            let mut effects = Effects::default();
            if let Some(kind) = older {
                effects.pending.start(
                    2,
                    obligation(
                        1,
                        vec![view.hands[1][0].id],
                        Card::new(suit, Rank::One),
                        if kind == HGroupConnectionKind::Prompt {
                            focus
                        } else {
                            view.hands[2][1].id
                        },
                        two,
                        kind,
                    ),
                );
            } else {
                // An ordinary clued play does not queue a fresh Bluff.
                effects.playing.insert(view.hands[1][0].id);
            }
            apply_bluff_effects(
                &entry,
                &view,
                &layout(&view),
                &connection_heights(&view, None),
                &effects.explicit,
                &effects.playing,
                &mut effects.pending,
                &mut effects.forced,
                &mut effects.signals,
            );
            assert_eq!(
                effects.signals.has_at_turn(4, HGroupMoveKind::Bluff),
                older.is_none()
            );
            if older.is_none() {
                let connection = effects.pending.iter().next().unwrap();
                assert_eq!(connection.cards, [blind]);
                assert_eq!(connection.focus_identity, two);
                assert_eq!(
                    connection_predecessors(&view, &connection_heights(&view, None), two),
                    [Card::new(suit, Rank::One)]
                );
            }
        }
    }
}

#[test]
fn contract_queued_self_bluff_is_also_illegal() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 3);
    let focus = view.hands[1][0].id;
    let three = Card::new(Suit::new(5), Rank::Three);
    view.hands[1][0].identity = Some(three);
    view.hands[1].last_mut().unwrap().identity = Some(Card::new(Suit::Green, Rank::One));
    let entry = clue_entry(1, Clue::Rank(Rank::Three), focus);
    for queued in [false, true] {
        let mut effects = Effects::default();
        if queued {
            effects.pending.start(
                2,
                obligation(
                    1,
                    vec![view.hands[1][1].id],
                    Card::new(Suit::Blue, Rank::One),
                    view.hands[2][0].id,
                    Card::new(Suit::Blue, Rank::Two),
                    HGroupConnectionKind::Finesse,
                ),
            );
        }
        apply_bluff_effects(
            &entry,
            &view,
            &layout(&view),
            &connection_heights(&view, None),
            &effects.explicit,
            &effects.playing,
            &mut effects.pending,
            &mut effects.forced,
            &mut effects.signals,
        );
        assert_eq!(
            effects.signals.has_at_turn(4, HGroupMoveKind::SelfBluff),
            !queued
        );
    }
}

#[test]
fn contract_sixth_suit_double_bluff_progresses_without_a_third_obligation() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 4);
    let focus = view.hands[3][0].id;
    let first = view.hands[1].last().unwrap().id;
    let second = view.hands[2].last().unwrap().id;
    let focus_identity = Card::new(Suit::new(5), Rank::Four);
    view.hands[3][0].identity = Some(focus_identity);
    view.hands[1].last_mut().unwrap().identity = Some(Card::new(Suit::Red, Rank::One));
    view.hands[2].last_mut().unwrap().identity = Some(Card::new(Suit::Red, Rank::Two));
    let entry = clue_entry(3, Clue::Suit(Suit::Blue), focus);
    let mut clue = interpretation(&view, focus, focus_identity, 3, Clue::Suit(Suit::Blue));
    clue.hypotheses.push(ClueInterpretationHypothesis {
        focus_identity,
        required_fix: None,
        loaded: false,
        connection_steps: vec![ClueConnectionStep {
            actor: PlayerId::new(1),
            cards: vec![first],
            expected: Card::new(Suit::new(5), Rank::One),
            kind: HGroupConnectionKind::Finesse,
        }],
    });
    let clues = [clue];
    let hands = layout(&view);
    let literal = facts(&view);
    let mut ctx = context(&view, &entry);
    ctx.after.hands = &hands;
    ctx.after.facts = &literal;
    let mut effects = Effects::default();
    apply_double_bluff_effects(&ctx, &view, &mut effects.borrow(&clues));
    assert!(effects.forced.contains(&first));
    assert!(!effects.forced.contains(&second));
    assert!(effects.signals.has_at_turn(4, HGroupMoveKind::DoubleBluff));
    effects.explicit.insert(focus);
    view.history.push(entry);
    for (turn, actor, card, identity) in [
        (5, 1, first, Card::new(Suit::Red, Rank::One)),
        (6, 2, second, Card::new(Suit::Red, Rank::Two)),
    ] {
        let played = ObservedHistoryEntry {
            turn,
            event: ObservedEvent::Played {
                player: PlayerId::new(actor),
                card,
                identity,
                successful: true,
            },
        };
        let hands = layout(&view);
        let literal = facts(&view);
        let mut ctx = context(&view, &played);
        ctx.after.hands = &hands;
        ctx.after.facts = &literal;
        connection_advance(&view, &mut ctx.after.dynamic_stack_heights, identity);
        apply_double_bluff_effects(&ctx, &view, &mut effects.borrow(&clues));
        assert!(
            effects
                .signals
                .has_at_turn(turn, HGroupMoveKind::DoubleBluff)
        );
        if turn == 5 {
            assert!(effects.forced.contains(&second));
        }
        assert!(effects.pending.iter().next().is_none());
        view.history.push(played);
        view.hands[actor as usize].retain(|held| held.id != card);
        view.play_stacks[0].push((card, identity));
    }
    assert!(!effects.forced.contains(&focus));
}

#[test]
fn contract_reordered_ids_layered_hidden_and_queued_domains_retain_lifecycle() {
    let mut view = snapshot(reordered(), 3);
    view.observer = PlayerId::new(1);
    let focus = view.hands[2][0].id;
    let first = view.hands[1].last().unwrap().id;
    let suffix = view.hands[1][2].id;
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    let five = Card::new(Suit::new(5), Rank::Five);
    assert_ne!(
        view.variant.standard_identity_id(three).unwrap().index(),
        three.index()
    );
    let d = LogicalDeductions::new(view.clone()).unwrap();
    let mut replay = replay_h_group(&d, HGroupProfile::Level(HGroupLevel::Level1));
    replay.pending_connections.start(
        1,
        obligation(
            1,
            vec![first, suffix],
            one,
            focus,
            five,
            HGroupConnectionKind::Finesse,
        ),
    );
    let mut later = obligation(
        1,
        vec![view.hands[1][0].id],
        three,
        focus,
        five,
        HGroupConnectionKind::Prompt,
    );
    later.step = 1;
    replay.pending_connections.start(2, later);
    replay.knowledge = build_convention_knowledge(&d, &replay);
    let notes = convention_card_inferences(&d, &replay);
    let first_note = notes.iter().find(|note| note.card == first).unwrap();
    assert!(
        first_note
            .identities
            .contains_id(view.variant.standard_identity_id(one).unwrap())
    );
    assert_eq!(first_note.promised_identity, Some(one));
    let queued_note = notes
        .iter()
        .find(|note| note.card == view.hands[1][0].id)
        .unwrap();
    assert_eq!(queued_note.identities, connection_domain(&view, three));
    assert_eq!(queued_note.promised_identity, Some(three));
    assert_eq!(queued_note.play_obligation, None);
    let schedule = ActionSchedule::from_replay(&view, &replay);
    assert!(
        schedule
            .plays_for(view.observer)
            .any(|play| play.identities == connection_domain(&view, one))
    );
    replay.pending_connections.advance_play(
        4,
        view.observer,
        first,
        Card::new(Suit::Red, Rank::One),
        true,
    );
    let active = replay
        .pending_connections
        .iter()
        .find(|p| p.expected == one)
        .unwrap();
    assert_eq!(active.cards, [suffix]);
    assert_eq!(active.expected, one);
    let mut heights = connection_heights(&view, None);
    connection_advance(&view, &mut heights, one);
    assert!(connection_playable(&view, &heights, three));
    assert_eq!(connection_predecessors(&view, &heights, five), [three]);
}

#[test]
fn contract_physical_clandestine_connection_precedes_bluff_with_reordered_ranks() {
    let mut view = snapshot(reordered(), 3);
    let one = Card::new(Suit::new(5), Rank::One);
    assert!(bluff_play_connects(&view, Clue::Suit(Suit::Red), one));
    assert!(bluff_play_connects(&view, Clue::Rank(Rank::Three), one));
    assert!(!bluff_play_connects(&view, Clue::Rank(Rank::Five), one));
    let focus = view.hands[2][0].id;
    let blind = view.hands[1].last().unwrap().id;
    view.hands[2][0].identity = Some(Card::new(Suit::new(5), Rank::Three));
    view.hands[1].last_mut().unwrap().identity = Some(Card::new(Suit::Red, Rank::One));
    let entry = clue_entry(2, Clue::Suit(Suit::Red), focus);
    let mut effects = Effects::default();
    apply_bluff_effects(
        &entry,
        &view,
        &layout(&view),
        &connection_heights(&view, None),
        &effects.explicit,
        &effects.playing,
        &mut effects.pending,
        &mut effects.forced,
        &mut effects.signals,
    );
    // The playable layer physically connects to the color clue. It cannot
    // collapse an apparent Clandestine line into a single Bluff response.
    assert!(!effects.signals.has_at_turn(4, HGroupMoveKind::Bluff));
    assert!(
        !effects
            .pending
            .iter()
            .any(|connection| connection.cards == [blind])
    );
}

#[test]
fn contract_sixth_suit_ejection_and_discharge_use_dynamic_stacks() {
    for discharge in [false, true] {
        let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 3);
        let focus = view.hands[2][0].id;
        let identity = Card::new(Suit::new(5), if discharge { Rank::One } else { Rank::Five });
        view.hands[2][0].identity = Some(identity);
        if discharge {
            view.play_stacks[5].push((CardId::new(40), identity));
        }
        let entry = clue_entry(2, Clue::Suit(Suit::Red), focus);
        let clues = [interpretation(
            &view,
            focus,
            identity,
            2,
            Clue::Suit(Suit::Red),
        )];
        let hands = layout(&view);
        let literal = facts(&view);
        let mut effects = Effects::default();
        apply_ejection_discharge_effects(
            &entry,
            &view,
            &hands,
            &CardSet::default(),
            &hands,
            &literal,
            &clues,
            &connection_heights(&view, None),
            &effects.explicit,
            &mut effects.invisible,
            &effects.moved,
            &mut effects.pending,
            &mut effects.forced,
            &mut effects.discard,
            &mut effects.signals,
            false,
        );
        let kind = if discharge {
            HGroupMoveKind::UnknownTrashDischarge
        } else {
            HGroupMoveKind::FiveColorEjection
        };
        assert!(effects.signals.has_at_turn(4, kind));
        let expected = view.hands[1][view.hands[1].len() - if discharge { 3 } else { 2 }].id;
        assert!(effects.forced.contains(&expected));
        assert_eq!(effects.discard.contains(&focus), discharge);
        if !discharge {
            assert_eq!(
                effects.signals.facts().known_identity(focus),
                Some(identity)
            );
        }
    }
}

#[test]
fn contract_demonstrated_ejection_keeps_all_physically_touched_fives_in_owner_domain() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 3);
    view.observer = PlayerId::new(2);
    let focus = view.hands[2][0].id;
    view.hands[2][0]
        .clues
        .add_positive_clue(Clue::Suit(Suit::Red));
    let d = LogicalDeductions::new(view.clone()).unwrap();
    let mut replay = replay_h_group(&d, HGroupProfile::Level(HGroupLevel::Level1));
    let mut effects = Effects::default();
    let entry = clue_entry(2, Clue::Suit(Suit::Red), focus);
    // Focus-only annotation emitted by demonstrated Ejection resolution when
    // physical touching cannot establish an exact hidden stack.
    push_signal(
        &mut effects.signals,
        &entry,
        PlayerId::new(0),
        Some(view.observer),
        HGroupMoveKind::FiveColorEjection,
        vec![focus],
        None,
    );
    let (signals, public_facts) = effects.signals.into_parts();
    replay.signals = signals;
    replay.cards.facts = public_facts;
    replay.knowledge = build_convention_knowledge(&d, &replay);
    let notes = convention_card_inferences(&d, &replay);
    let note = notes.iter().find(|note| note.card == focus).unwrap();
    let expected = connection_domain(&view, Card::new(Suit::Red, Rank::Five)).union(
        connection_domain(&view, Card::new(Suit::new(5), Rank::Five)),
    );
    assert_eq!(note.identities, expected);
    assert_eq!(note.promised_identity, None);
}

#[test]
fn contract_priority_load_maps_physical_claims_to_active_ids() {
    let mut view = snapshot(reordered(), 3);
    view.observer = PlayerId::new(1);
    let played = view.hands[0][0].id;
    let declined = view.hands[0][1].id;
    let one = Card::new(Suit::new(5), Rank::One);
    let connector = Card::new(Suit::new(5), Rank::Three);
    view.hands[0][0]
        .clues
        .add_positive_clue(Clue::Rank(Rank::One));
    view.hands[0][1]
        .clues
        .add_positive_clue(Clue::Rank(Rank::One));
    let entry = ObservedHistoryEntry {
        turn: 4,
        event: ObservedEvent::Played {
            player: PlayerId::new(0),
            card: played,
            identity: one,
            successful: true,
        },
    };
    let ctx = context(&view, &entry);
    let mut effects = Effects::default();
    // Public exact claims establish a deliberate choice without hidden truth.
    for (card, id) in [(played, one), (declined, Card::new(Suit::Red, Rank::One))] {
        effects.explicit.insert(card);
        push_signal(
            &mut effects.signals,
            &ObservedHistoryEntry {
                turn: 2,
                event: entry.event.clone(),
            },
            PlayerId::new(0),
            Some(PlayerId::new(0)),
            HGroupMoveKind::PlayClue,
            vec![card],
            Some(id),
        );
    }
    // The played identity is publicly known; the alternate has equal rank and
    // a later hand position, exercising Priority's existing age ordering.
    let mut literal = ctx.before.facts.clone();
    literal[played.index()].record_physical(Clue::Suit(Suit::Red).into(), true);
    // Both suits touch red in this custom universe. Use live exact knowledge
    // for the played card and an artificial domain containing only safe ones.
    let mut ctx = ctx;
    ctx.before.facts = literal;
    apply_priority_effects(
        &ctx,
        &view,
        &effects.explicit,
        &effects.pending,
        &mut effects.forced,
        &mut effects.signals,
    );
    assert!(effects.signals.has_at_turn(4, HGroupMoveKind::Priority));
    let priority = effects
        .signals
        .of_kind(HGroupMoveKind::Priority)
        .last()
        .unwrap();
    assert_eq!(priority.identity, Some(connector));
    let promised = priority.cards[0];
    let load = clue_entry(1, Clue::Rank(Rank::Three), view.hands[1][0].id);
    let excluded = retract_loaded_priority(
        &load,
        &view,
        &layout(&view),
        &mut effects.forced,
        &mut effects.signals,
    );
    assert_eq!(excluded, connection_domain(&view, connector));
    assert!(!effects.forced.contains(&promised));
    assert_eq!(
        connection_bit(&view, connector),
        1 << view
            .variant
            .standard_identity_id(connector)
            .unwrap()
            .index()
    );
}

#[test]
fn contract_hidden_truth_does_not_change_owner_connection_domains() {
    let mut baseline = None;
    for hidden in [
        None,
        Some(Card::new(Suit::Red, Rank::Five)),
        Some(Card::new(Suit::new(5), Rank::One)),
    ] {
        let mut view = snapshot(reordered(), 3);
        let own = view.hands[0].last().unwrap().id;
        view.hands[0].last_mut().unwrap().identity = hidden;
        assert_eq!(HistoricalView::new(&view, 4).identity(own), None);
        let focus = view.hands[2][0].id;
        let expected = Card::new(Suit::new(5), Rank::One);
        let d = LogicalDeductions::new(view.clone()).unwrap();
        let mut replay = replay_h_group(&d, HGroupProfile::Level(HGroupLevel::Level1));
        replay.pending_connections.start(
            2,
            obligation(
                0,
                vec![own],
                expected,
                focus,
                Card::new(Suit::new(5), Rank::Three),
                HGroupConnectionKind::Finesse,
            ),
        );
        replay.knowledge = build_convention_knowledge(&d, &replay);
        let notes = convention_card_inferences(&d, &replay);
        let note = notes.iter().find(|note| note.card == own).unwrap();
        let observed = (
            note.identities,
            note.promised_identity,
            note.play_obligation,
        );
        if let Some(previous) = baseline {
            assert_eq!(observed, previous);
        } else {
            baseline = Some(observed);
        }
        assert_eq!(note.identities, connection_domain(&view, expected));
    }
}

#[test]
fn contract_hidden_truthful_connector_uses_dated_public_promise() {
    let mut view = snapshot(reordered(), 3);
    let own = view.hands[0].last().unwrap().id;
    let one = Card::new(Suit::new(5), Rank::One);
    let mut effects = Effects::default();
    effects.playing.insert(own);
    let mut entry = clue_entry(2, Clue::Rank(Rank::One), own);
    entry.turn = 2;
    push_signal(
        &mut effects.signals,
        &entry,
        PlayerId::new(2),
        Some(view.observer),
        HGroupMoveKind::PlayClue,
        vec![own],
        Some(one),
    );
    for hidden in [None, Some(Card::new(Suit::Red, Rank::Five)), Some(one)] {
        view.hands[0].last_mut().unwrap().identity = hidden;
        assert!(super::super::bluff::bluff_connector_is_promised(
            &view,
            &layout(&view),
            &effects.playing,
            &effects.pending,
            effects.signals.facts(),
            4,
            one,
            None
        ));
        assert!(!super::super::bluff::bluff_connector_is_promised(
            &view,
            &layout(&view),
            &effects.playing,
            &effects.pending,
            effects.signals.facts(),
            1,
            one,
            None
        ));
    }
}

#[test]
fn contract_complete_double_finesse_precedes_apparent_three_bluff() {
    for (variant, suit) in [
        (VariantSpec::no_variant(), Suit::Red),
        (
            VariantSpec::from_catalog(VariantId::Rainbow6Suits),
            Suit::new(5),
        ),
    ] {
        for complete in [false, true] {
            let mut view = snapshot(variant.clone(), 3);
            let focus = view.hands[2][0].id;
            view.hands[2][0].identity = Some(Card::new(suit, Rank::Three));
            view.hands[1].last_mut().unwrap().identity = Some(Card::new(Suit::Green, Rank::One));
            let entry = clue_entry(2, Clue::Rank(Rank::Three), focus);
            let mut clue = interpretation(
                &view,
                focus,
                Card::new(suit, Rank::Three),
                2,
                Clue::Rank(Rank::Three),
            );
            if complete {
                clue.hypotheses.push(ClueInterpretationHypothesis {
                    focus_identity: Card::new(suit, Rank::Three),
                    connection_steps: [Rank::One, Rank::Two]
                        .into_iter()
                        .map(|rank| ClueConnectionStep {
                            actor: PlayerId::new(1),
                            cards: vec![view.hands[1][0].id],
                            expected: Card::new(suit, rank),
                            kind: HGroupConnectionKind::Finesse,
                        })
                        .collect(),
                    required_fix: None,
                    loaded: false,
                });
            }
            let hands = layout(&view);
            let literal = facts(&view);
            let mut ctx = context(&view, &entry);
            ctx.after.hands = &hands;
            ctx.after.facts = &literal;
            let mut effects = Effects::default();
            push_signal(
                &mut effects.signals,
                &entry,
                PlayerId::new(0),
                Some(PlayerId::new(2)),
                HGroupMoveKind::PlayClue,
                vec![focus],
                None,
            );
            apply_intermediate_bluff_effects(&ctx, &view, &mut effects.borrow(&[clue]));
            assert_eq!(
                effects.signals.has_at_turn(4, HGroupMoveKind::ThreeBluff),
                !complete
            );
            assert_eq!(
                effects.signals.has_at_turn(4, HGroupMoveKind::Bluff),
                !complete
            );
        }
    }
}

#[test]
fn contract_hidden_finesse_after_prompt_uses_reordered_successor_domain() {
    let mut view = snapshot(reordered(), 3);
    view.observer = PlayerId::new(1);
    let prompt = view.hands[1][0].id;
    let blind = view.hands[1].last().unwrap().id;
    let focus = view.hands[2][0].id;
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    let five = Card::new(Suit::new(5), Rank::Five);
    let d = LogicalDeductions::new(view.clone()).unwrap();
    let mut replay = replay_h_group(&d, HGroupProfile::Level(HGroupLevel::Level1));
    replay.pending_connections.start(
        1,
        obligation(
            1,
            vec![prompt],
            one,
            focus,
            five,
            HGroupConnectionKind::Prompt,
        ),
    );
    let mut hidden = obligation(
        1,
        vec![blind],
        three,
        focus,
        five,
        HGroupConnectionKind::Finesse,
    );
    hidden.step = 1;
    replay.pending_connections.start(1, hidden);
    replay.knowledge = build_convention_knowledge(&d, &replay);
    let notes = convention_card_inferences(&d, &replay);
    let note = notes.iter().find(|note| note.card == blind).unwrap();
    assert_eq!(note.identities, connection_domain(&view, three));
    assert_eq!(note.promised_identity, Some(three));
    assert_eq!(note.play_obligation, None);
    replay
        .pending_connections
        .advance_play(4, view.observer, prompt, one, true);
    view.current_player = view.observer;
    view.play_stacks[0].push((prompt, one));
    view.hands[1].retain(|card| card.id != prompt);
    replay.hands = layout(&view);
    let d = LogicalDeductions::new(view).unwrap();
    replay.knowledge = build_convention_knowledge(&d, &replay);
    let notes = convention_card_inferences(&d, &replay);
    let note = notes.iter().find(|note| note.card == blind).unwrap();
    assert_eq!(note.identities, connection_domain(d.view(), three));
    assert_eq!(
        note.play_obligation,
        Some(HGroupPlayObligation::Connection(
            HGroupConnectionKind::Finesse
        ))
    );
}

// V5 contracts reuse these physical snapshots; they establish no variant
// convention pack. Context's documented precedence is unchanged.
#[test]
fn contract_v5_context_dated_active_connector_and_observer_negative() {
    for (variant, suit, next) in [
        (VariantSpec::no_variant(), Suit::Red, Rank::Two),
        (reordered(), Suit::new(5), Rank::Three),
    ] {
        let mut view = snapshot(variant, 3);
        let focus = view.hands[1][0].id;
        let identity = Card::new(suit, next);
        view.hands[1][0].identity = Some(identity);
        // Current stacks are later than the event being interpreted.
        view.play_stacks[0] = vec![(CardId::new(50), Card::new(suit, Rank::One))];
        let entry = clue_entry(1, Clue::Rank(next), focus);
        let hands = layout(&view);
        let facts = facts(&view);
        let mut context = context(&view, &entry);
        context.before.dynamic_stack_heights.fill(0);
        context.after.hands = &hands;
        context.after.facts = &facts;
        let mut effects = Effects::default();
        crate::h_group::recognition::apply_context_effects(
            &context,
            &view,
            &mut effects.borrow(&[]),
        );
        let signal = effects
            .signals
            .iter()
            .find(|s| s.kind == HGroupMoveKind::Context)
            .unwrap();
        assert_eq!(signal.identity, Some(identity));
        let mut claims = ConventionFacts::default();
        claims.apply_identity_effect(signal);
        assert_eq!(claims.known_identity(focus), Some(identity));

        // A future reveal of the observer's own card must not establish Context.
        let hidden = view.hands[0][0].id;
        view.hands[0][0].identity = Some(identity);
        let hidden_entry = clue_entry(0, Clue::Rank(next), hidden);
        let mut context = self::context(&view, &hidden_entry);
        context.after.hands = &hands;
        let mut effects = Effects::default();
        crate::h_group::recognition::apply_context_effects(
            &context,
            &view,
            &mut effects.borrow(&[]),
        );
        assert!(effects.signals.iter().next().is_none());
    }
}

#[test]
fn contract_v5_context_active_focus_inversion_preserves_existing_connection_and_fix() {
    for variant in [VariantSpec::no_variant(), reordered()] {
        let mut view = snapshot(variant, 3);
        let old = view.hands[1][0].id;
        let new = view.hands[1][1].id;
        view.hands[1].truncate(2);
        // Rank 1 is playable in both universes; rank 5 is not.
        view.hands[1][0].clues = ClueFacts::default();
        let mut entry = clue_entry(1, Clue::Suit(Suit::Red), new);
        if let ObservedEvent::Clued { touched, .. } = &mut entry.event {
            *touched = vec![old, new];
        }
        let mut context = context(&view, &entry);
        // Active-ID fact masks deliberately exclude a directly playable old focus.
        let five = Card::new(Suit::Red, Rank::Five);
        context.before.facts[old.index()].add_positive_clue(Clue::Rank(Rank::Five));
        for guard in [0, 1, 2] {
            let mut effects = Effects::default();
            let clues = vec![interpretation(&view, old, five, 1, Clue::Suit(Suit::Red))];
            if guard == 1 {
                effects.pending.start(
                    entry.turn,
                    obligation(
                        2,
                        vec![view.hands[2][0].id],
                        Card::new(Suit::Red, Rank::One),
                        old,
                        five,
                        HGroupConnectionKind::Finesse,
                    ),
                );
            }
            if guard == 2 {
                push_signal(
                    &mut effects.signals,
                    &entry,
                    PlayerId::new(0),
                    Some(PlayerId::new(1)),
                    HGroupMoveKind::FixClue,
                    vec![old],
                    None,
                );
            }
            crate::h_group::recognition::apply_context_effects(
                &context,
                &view,
                &mut effects.borrow(&clues),
            );
            assert_eq!(
                effects
                    .signals
                    .has_at_turn(entry.turn, HGroupMoveKind::FocusInversion),
                guard == 0
            );
            assert_eq!(effects.forced.contains(&new), guard == 0);
        }
    }
}

#[test]
fn contract_v5_duplication_active_old_duplicate_proof_and_cancellation() {
    for (variant, suit) in [
        (VariantSpec::no_variant(), Suit::Red),
        (reordered(), Suit::new(5)),
    ] {
        let mut view = snapshot(variant, 3);
        let played = view.hands[1][0].id;
        let duplicate = view.hands[0][0].id;
        let cover = view.hands[2][0].id;
        let identity = Card::new(suit, Rank::One);
        view.hands[0][0]
            .clues
            .add_positive_clue(Clue::Rank(Rank::One));
        if suit == Suit::Red {
            view.hands[0][0]
                .clues
                .add_positive_clue(Clue::Suit(Suit::Red));
        }
        view.hands[2][0].identity = Some(Card::new(Suit::Red, Rank::One));
        let entry = ObservedHistoryEntry {
            turn: 4,
            event: ObservedEvent::Played {
                player: PlayerId::new(1),
                card: played,
                identity,
                successful: true,
            },
        };
        let mut hands = layout(&view);
        hands[1].retain(|card| *card != played);
        let facts = facts(&view);
        let mut context = context(&view, &entry);
        context.after.hands = &hands;
        context.after.facts = &facts;
        for old in [false, true] {
            let mut origin = interpretation(&view, played, identity, 1, Clue::Rank(Rank::One));
            origin.turn = 2;
            if old {
                origin.previously_gotten.push(duplicate);
            }
            let mut trash = interpretation(&view, duplicate, identity, 0, Clue::Rank(Rank::One));
            trash.turn = 1;
            let clues = vec![trash, origin];
            let mut effects = Effects::default();
            effects.explicit.extend([duplicate, cover]);
            effects.playing.insert(played);
            effects.pending.start(
                2,
                obligation(
                    1,
                    vec![played],
                    identity,
                    played,
                    identity,
                    HGroupConnectionKind::Finesse,
                ),
            );
            let mut trash_entry = entry.clone();
            trash_entry.turn = 1;
            push_signal(
                &mut effects.signals,
                &trash_entry,
                PlayerId::new(2),
                Some(PlayerId::new(0)),
                HGroupMoveKind::TrashChopMove,
                vec![duplicate],
                None,
            );
            crate::h_group::recognition::apply_duplication_effects(
                &context,
                &view,
                &mut effects.borrow(&clues),
            );
            assert_eq!(
                effects
                    .signals
                    .has_at_turn(4, HGroupMoveKind::TimeTravelChopMove),
                old
            );
            assert_eq!(effects.playing.contains(&played), !old);
            assert_eq!(
                effects.pending.iter().any(|c| effects.pending.is_active(c)),
                !old
            );
        }
    }
}

#[test]
fn contract_v5_duplication_extra_value_uses_origin_heights() {
    for (variant, suit) in [
        (VariantSpec::no_variant(), Suit::Red),
        (reordered(), Suit::new(5)),
    ] {
        let mut view = snapshot(variant, 3);
        let played = view.hands[1][0].id;
        let duplicate = view.hands[2][0].id;
        let extra = view.hands[2][1].id;
        let identity = Card::new(suit, Rank::One);
        view.hands[2][0].identity = Some(identity);
        view.hands[2][1].identity = Some(identity);
        // This later public play makes all remaining copies trash now, but
        // they were live at the originating clue. This is a dating invariant.
        view.history.push(ObservedHistoryEntry {
            turn: 3,
            event: ObservedEvent::Played {
                player: PlayerId::new(2),
                card: CardId::new(40),
                identity,
                successful: true,
            },
        });
        let entry = ObservedHistoryEntry {
            turn: 4,
            event: ObservedEvent::Played {
                player: PlayerId::new(1),
                card: played,
                identity,
                successful: true,
            },
        };
        let mut hands = layout(&view);
        hands[1].retain(|card| *card != played);
        let facts = facts(&view);
        let mut context = context(&view, &entry);
        context.after.hands = &hands;
        context.after.facts = &facts;
        let mut origin = interpretation(&view, played, identity, 1, Clue::Rank(Rank::One));
        origin.turn = 2;
        origin.previously_gotten.push(duplicate);
        origin.new_non_focus.push(extra);
        // The retained standard stack field cannot describe the sixth suit;
        // dynamic historical reconstruction must be the source of truth.
        origin.stack_heights = [5; 5];
        let mut effects = Effects::default();
        effects.explicit.insert(duplicate);
        crate::h_group::recognition::apply_duplication_effects(
            &context,
            &view,
            &mut effects.borrow(&[origin]),
        );
        assert!(
            effects
                .signals
                .has_at_turn(4, HGroupMoveKind::DuplicitousValue)
        );
        assert!(
            !effects
                .signals
                .has_at_turn(4, HGroupMoveKind::TimeTravelChopMove)
        );
    }
}

#[test]
fn contract_v5_elimination_active_copy_count_resolution_and_constraints() {
    for (variant, suit, copies) in [
        (VariantSpec::no_variant(), Suit::Red, 3),
        (reordered(), Suit::new(5), 4),
    ] {
        let mut view = snapshot(variant, 3);
        let identity = Card::new(suit, Rank::One);
        let discarded = view.hands[0][0].id;
        let candidate = view.hands[0][1].id;
        view.hands[0].truncate(2);
        // Leave exactly one physical copy after this discard. Reordered
        // variants deliberately disagree with Rank::copies().
        for turn in 0..copies - 2 {
            view.history.push(ObservedHistoryEntry {
                turn,
                event: ObservedEvent::Discarded {
                    player: PlayerId::new(2),
                    card: CardId::new(40 + turn as usize),
                    identity,
                },
            });
        }
        let entry = ObservedHistoryEntry {
            turn: 4,
            event: ObservedEvent::Discarded {
                player: PlayerId::new(0),
                card: discarded,
                identity,
            },
        };
        let mut hands = layout(&view);
        hands[0].retain(|card| *card != discarded);
        let facts = facts(&view);
        let mut context = context(&view, &entry);
        context.after.hands = &hands;
        context.after.facts = &facts;
        let mut effects = Effects::default();
        crate::h_group::recognition::apply_elimination_effects(
            &entry,
            &view,
            &hands,
            &context.after.dynamic_stack_heights,
            1,
            &mut effects.signals,
        );
        crate::h_group::recognition::apply_elimination_resolution_effects(
            &context,
            &view,
            &mut effects.borrow(&[]),
        );
        assert!(effects.signals.has_at_turn(4, HGroupMoveKind::Elimination));
        assert!(effects.moved.contains(&candidate));
        assert!(effects.forced.contains(&candidate));
        // Build only the consumer seed through the No Variant-era reducer.
        // Sixth-suit historical discards still hit its retained public_removed
        // array; the production migration does not claim that boundary closed.
        let mut seed = view.clone();
        seed.history.clear();
        let deductions = LogicalDeductions::new(seed).unwrap();
        let mut replay = replay_h_group(&deductions, HGroupProfile::Level(HGroupLevel::Level1));
        replay.cards.facts = effects.signals.facts().clone();
        let inferred = HGroupInferences::default();
        let constraints = crate::h_group::constraint_graph::ConventionConstraintGraph::from_replay(
            &deductions,
            &replay,
            &inferred,
        )
        .into_belief_constraints();
        assert!(
            constraints
                .factors
                .iter()
                .flatten()
                .any(|branch| branch.contains(&(candidate, connection_domain(&view, identity))))
        );

        // A literal exclusion retracts the claim instead of forcing an
        // impossible constraint, even in the reordered active universe.
        context.before.facts[candidate.index()].add_negative_clue(Clue::Rank(Rank::One));
        context.after.facts = &context.before.facts;
        crate::h_group::recognition::apply_elimination_resolution_effects(
            &context,
            &view,
            &mut effects.borrow(&[]),
        );
        assert!(effects.signals.facts().identity_claims().is_empty());
    }
}

#[test]
fn contract_v5_elimination_visibility_and_exhausted_copy_negative() {
    for (variant, suit, copies) in [
        (VariantSpec::no_variant(), Suit::Red, 3),
        (reordered(), Suit::new(5), 4),
    ] {
        for visible in [false, true] {
            let mut view = snapshot(variant.clone(), 3);
            let identity = Card::new(suit, Rank::One);
            let discarded = view.hands[1][0].id;
            // Opponent visible copy blocks Elimination. The observer's own
            // current identity, even if injected, cannot be earlier evidence.
            view.hands[if visible { 2 } else { 0 }][0].identity = Some(identity);
            let entry = ObservedHistoryEntry {
                turn: 4,
                event: ObservedEvent::Discarded {
                    player: PlayerId::new(1),
                    card: discarded,
                    identity,
                },
            };
            let mut hands = layout(&view);
            hands[1].retain(|card| *card != discarded);
            let heights = vec![0; view.variant.stack_suits().len()];
            let mut signals = ConventionJournal::default();
            crate::h_group::recognition::apply_elimination_effects(
                &entry,
                &view,
                &hands,
                &heights,
                1,
                &mut signals,
            );
            assert_eq!(
                signals.has_at_turn(4, HGroupMoveKind::Elimination),
                !visible
            );
            for turn in 0..copies - 1 {
                view.history.push(ObservedHistoryEntry {
                    turn,
                    event: ObservedEvent::Discarded {
                        player: PlayerId::new(2),
                        card: CardId::new(40 + turn as usize),
                        identity,
                    },
                });
            }
            let mut signals = ConventionJournal::default();
            crate::h_group::recognition::apply_elimination_effects(
                &entry,
                &view,
                &hands,
                &heights,
                1,
                &mut signals,
            );
            assert!(!signals.has_at_turn(4, HGroupMoveKind::Elimination));
        }
    }
}
