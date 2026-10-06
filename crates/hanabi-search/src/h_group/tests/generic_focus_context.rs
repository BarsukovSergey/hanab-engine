// Physical and epistemic contracts, not invented strategy authorities. The
// reviewed No Variant expectations remain in learning_rules.rs and replay.rs
// (Tempo/TCM, 5 Stall/5CM, and recipient-relative Gentleman transfers).
use super::*;
use crate::h_group::identity::{connection_domain, connection_heights};
use hanabi_core::{ClueColorId, ClueRank, IdentitySpec, VariantId, VariantSpec};

fn snapshot(variant: VariantSpec, players: u8) -> PlayerView {
    let mut view = FullState::new_standard(players, standard_deck())
        .unwrap()
        .view_for(PlayerId::new(0))
        .unwrap();
    view.play_stacks = vec![vec![]; variant.stack_suits().len()];
    view.variant = variant;
    view.turn = 2;
    view.history.clear();
    for card in view.hands.iter_mut().flatten() {
        card.identity = None;
    }
    view
}

fn irregular() -> VariantSpec {
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
                clue_colors: vec![ClueColorId::new(suit.value())],
                clue_ranks: vec![ClueRank::new(rank.value())],
            })
        })
        .collect();
    VariantSpec::new(
        "V3 physical contract",
        suits,
        ranks,
        vec![ClueColorId::new(0), ClueColorId::new(5)],
        vec![ClueRank::new(1), ClueRank::new(3), ClueRank::new(5)],
        identities,
    )
    .unwrap()
}

fn advance(view: &mut PlayerView, suit: Suit, ranks: &[Rank]) {
    let index = view
        .variant
        .stack_suits()
        .iter()
        .position(|candidate| *candidate == suit)
        .unwrap();
    view.play_stacks[index] = ranks
        .iter()
        .enumerate()
        .map(|(index, rank)| (CardId::new(40 + index), Card::new(suit, *rank)))
        .collect();
}

fn context<'a>(
    view: &'a PlayerView,
    entry: &'a hanabi_core::ObservedHistoryEntry,
) -> HGroupTurnContext<'a> {
    let hands: Vec<Vec<CardId>> = view
        .hands
        .iter()
        .map(|hand| hand.iter().map(|card| card.id).collect())
        .collect();
    let mut facts = vec![ClueFacts::default(); 64];
    for card in view.hands.iter().flatten() {
        facts[card.id.index()] = card.clues;
    }
    let heights = connection_heights(view, None);
    // All events in these contracts are clues or discards: heights do not change.
    HGroupTurnContext {
        entry,
        historical: HistoricalView::new(view, entry.turn),
        before: HGroupTurnSnapshot {
            hands,
            facts,
            stack_heights: [0; 5],
            dynamic_stack_heights: heights.clone(),
            clue_tokens: 0,
            deck_size: 30,
            early_game: false,
            protected: CardSet::default(),
            already_playing: CardSet::default(),
            forced_playable: CardSet::default(),
            older_play_obligations: CardSet::default(),
        },
        // Borrowed after tables are supplied by the caller below.
        after: HGroupTurnView {
            hands: &[],
            facts: &[],
            stack_heights: [0; 5],
            dynamic_stack_heights: heights,
            clue_tokens: 0,
            deck_size: 30,
            early_game: false,
        },
        actor_before: ActorBeliefBefore::default(),
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

fn clue_entry(target: u8, clue: Clue, touched: Vec<CardId>) -> hanabi_core::ObservedHistoryEntry {
    hanabi_core::ObservedHistoryEntry {
        turn: 2,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(if target == 0 { 2 } else { 0 }),
            target: PlayerId::new(target),
            clue,
            touched,
            untouched: vec![],
        },
    }
}

#[test]
fn v3_focus_context_tempo_precedes_accounted_trash_chop_move() {
    let mut view = snapshot(VariantSpec::no_variant(), 3);
    let focus = view.hands[1][2].id;
    let duplicate = view.hands[2][0].id;
    let blue_two = Card::new(Suit::Blue, Rank::Two);
    view.hands[2][0].identity = Some(blue_two);
    view.hands[1][2]
        .clues
        .add_positive_clue(Clue::Rank(Rank::Two));
    view.history.push(hanabi_core::ObservedHistoryEntry {
        turn: 1,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(2),
            target: PlayerId::new(1),
            clue: Clue::Rank(Rank::Two),
            touched: vec![focus],
            untouched: vec![],
        },
    });
    advance(&mut view, Suit::Blue, &[Rank::One]);
    let entry = clue_entry(1, Clue::Suit(Suit::Blue), vec![focus]);
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let mut facts = ctx.before.facts.clone();
    facts[focus.index()].add_positive_clue(Clue::Suit(Suit::Blue));
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    let setup = || {
        let mut effects = Effects::default();
        effects.explicit.extend([focus, duplicate]);
        effects
    };
    let mut chop_only = setup();
    recognition::apply_chop_move_effects(&ctx, &view, &mut chop_only.borrow(&[]));
    assert!(
        chop_only
            .signals
            .has_at_turn(2, HGroupMoveKind::TrashChopMove)
    );
    let mut effects = setup();
    recognition::apply_tempo_effects(&ctx, &view, &mut effects.borrow(&[]));
    recognition::apply_chop_move_effects(&ctx, &view, &mut effects.borrow(&[]));
    assert!(effects.signals.has_at_turn(2, HGroupMoveKind::TempoClue));
    assert!(
        !effects
            .signals
            .has_at_turn(2, HGroupMoveKind::TrashChopMove)
    );
    assert!(
        effects
            .signals
            .has_at_turn(2, HGroupMoveKind::TempoClueChopMove)
    );
    assert!(effects.pending.is_empty());
}

#[test]
fn v3_focus_context_five_stall_precedes_five_chop_move() {
    let view = snapshot(VariantSpec::no_variant(), 3);
    let focus = view.hands[1][1].id;
    let entry = clue_entry(1, Clue::Rank(Rank::Five), vec![focus]);
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let mut facts = ctx.before.facts.clone();
    facts[focus.index()].add_positive_clue(Clue::Rank(Rank::Five));
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    for stall in [false, true] {
        let mut effects = Effects::default();
        if stall {
            push_signal(
                &mut effects.signals,
                &entry,
                PlayerId::new(0),
                Some(PlayerId::new(1)),
                HGroupMoveKind::FiveStall,
                vec![focus],
                None,
            );
        }
        recognition::apply_chop_move_effects(&ctx, &view, &mut effects.borrow(&[]));
        assert_eq!(
            effects.signals.has_at_turn(2, HGroupMoveKind::FiveChopMove),
            !stall
        );
        assert_eq!(effects.moved.is_empty(), stall);
        assert!(effects.pending.is_empty());
    }
}

#[test]
fn v3_focus_context_custom_trash_chop_is_protection_and_hidden_truth_independent() {
    for variant in [
        VariantSpec::from_catalog(VariantId::Black6Suits),
        irregular(),
    ] {
        for hidden in [
            None,
            Some(Card::new(Suit::new(5), Rank::Five)),
            Some(Card::new(Suit::Red, Rank::One)),
        ] {
            let mut view = snapshot(variant.clone(), 3);
            for suit in variant.stack_suits() {
                advance(&mut view, *suit, &[Rank::One]);
            }
            let touched = view.hands[0][2].id;
            let chop = view.hands[0][0].id;
            view.hands[0][2].identity = hidden;
            view.hands[0][2]
                .clues
                .add_positive_clue(Clue::Rank(Rank::One));
            let entry = clue_entry(0, Clue::Rank(Rank::One), vec![touched]);
            let mut ctx = context(&view, &entry);
            let hands = ctx.before.hands.clone();
            let facts = ctx.before.facts.clone();
            ctx.after.hands = &hands;
            ctx.after.facts = &facts;
            let mut effects = Effects::default();
            recognition::apply_chop_move_effects(&ctx, &view, &mut effects.borrow(&[]));
            assert!(
                effects
                    .signals
                    .has_at_turn(2, HGroupMoveKind::TrashChopMove)
            );
            assert!(effects.moved.contains(&chop));
            assert!(!effects.invisible.contains(&chop));
            assert!(!effects.playing.contains(&chop));
            assert!(effects.signals.facts().known_identity(chop).is_none());
            assert!(effects.pending.is_empty());
        }
    }
}

#[test]
fn v3_focus_context_sixth_suit_tempo_uses_physical_rainbow_facts() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Rainbow6Suits), 3);
    let focus = view.hands[1][2].id;
    view.hands[1][2]
        .clues
        .add_positive_clue(Clue::Rank(Rank::Three));
    view.history.push(hanabi_core::ObservedHistoryEntry {
        turn: 1,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(2),
            target: PlayerId::new(1),
            clue: Clue::Rank(Rank::Three),
            touched: vec![focus],
            untouched: vec![],
        },
    });
    advance(&mut view, Suit::Red, &[Rank::One, Rank::Two]);
    advance(&mut view, Suit::new(5), &[Rank::One, Rank::Two]);
    let entry = clue_entry(1, Clue::Suit(Suit::Red), vec![focus]);
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let mut facts = ctx.before.facts.clone();
    facts[focus.index()].add_positive_clue(Clue::Suit(Suit::Red));
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    let domain = IdentitySet::from_bits(facts[focus.index()].identity_mask_in(&view.variant));
    assert!(
        domain.contains_id(
            view.variant
                .standard_identity_id(Card::new(Suit::new(5), Rank::Three))
                .unwrap()
        )
    );
    for height in [2, 1] {
        ctx.before.dynamic_stack_heights[5] = height;
        let mut effects = Effects::default();
        effects.explicit.insert(focus);
        recognition::apply_tempo_effects(&ctx, &view, &mut effects.borrow(&[]));
        assert_eq!(
            effects.signals.has_at_turn(2, HGroupMoveKind::TempoClue),
            height == 2
        );
    }
}

#[test]
fn v3_focus_context_sixth_suit_transfer_layer_progression_and_baton_guard() {
    for (rank, layered, expected) in [
        (Rank::Three, false, Some(HGroupMoveKind::GentlemansDiscard)),
        (
            Rank::Three,
            true,
            Some(HGroupMoveKind::LayeredGentlemansDiscard),
        ),
        (Rank::Five, false, Some(HGroupMoveKind::BatonDiscard)),
        (Rank::Five, true, None),
    ] {
        let mut view = snapshot(irregular(), 3);
        let suit = Suit::new(5);
        let identity = Card::new(suit, rank);
        advance(&mut view, suit, &[Rank::One]);
        let discarded = view.hands[0][0].id;
        let newest = view.hands[1].len() - 1;
        let position = newest - usize::from(layered);
        let target = view.hands[1][position].id;
        view.hands[1][position].identity = Some(identity);
        if layered {
            view.hands[1][newest].identity = Some(if rank == Rank::Three {
                Card::new(Suit::Red, Rank::One)
            } else {
                Card::new(suit, Rank::Three)
            });
        }
        view.history.push(hanabi_core::ObservedHistoryEntry {
            turn: 1,
            event: ObservedEvent::Clued {
                giver: PlayerId::new(2),
                target: PlayerId::new(0),
                clue: Clue::Rank(rank),
                touched: vec![discarded],
                untouched: vec![],
            },
        });
        let entry = hanabi_core::ObservedHistoryEntry {
            turn: 2,
            event: ObservedEvent::Discarded {
                player: PlayerId::new(0),
                card: discarded,
                identity,
            },
        };
        let mut ctx = context(&view, &entry);
        let hands = ctx.before.hands.clone();
        let facts = ctx.before.facts.clone();
        ctx.after.hands = &hands;
        ctx.after.facts = &facts;
        ctx.actor_before.discarded_identity = Some(identity);
        let mut effects = Effects::default();
        let borrowed = effects.borrow(&[]);
        recognition::apply_transfer_effects(
            &ctx,
            &view,
            borrowed.explicitly_clued,
            borrowed.invisibly_clued,
            borrowed.already_playing,
            borrowed.pending,
            borrowed.signals,
        );
        if let Some(kind) = expected {
            assert!(
                effects.signals.has_at_turn(2, kind),
                "{rank:?}, layered={layered}, expected={kind:?}"
            );
            assert!(effects.invisible.contains(&target));
            assert_eq!(effects.playing.contains(&target), rank == Rank::Three);
        } else {
            assert!(effects.signals.iter().next().is_none());
        }
    }
}

#[test]
fn v3_focus_context_six_player_emergency_order_uses_actual_seats() {
    let mut view = snapshot(VariantSpec::from_catalog(VariantId::Black6Suits), 6);
    view.observer = PlayerId::new(5);
    for suit in view.variant.stack_suits().to_vec() {
        advance(&mut view, suit, &[Rank::One]);
    }
    for card in &mut view.hands[5][..3] {
        card.clues.add_positive_clue(Clue::Rank(Rank::One));
    }
    let discarded = view.hands[5][0].id;
    let entry = hanabi_core::ObservedHistoryEntry {
        turn: 2,
        event: ObservedEvent::Discarded {
            player: PlayerId::new(5),
            card: discarded,
            identity: Card::new(Suit::new(5), Rank::One),
        },
    };
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let facts = ctx.before.facts.clone();
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    let mut effects = Effects::default();
    effects
        .explicit
        .extend(view.hands[5][..3].iter().map(|card| card.id));
    recognition::apply_emergency_discard_effects(&ctx, &view, &mut effects.borrow(&[]), true);
    let signal = effects
        .signals
        .iter()
        .find(|signal| signal.kind == HGroupMoveKind::TrashOrderChopMove)
        .unwrap();
    // Three-card hands, newest first: two skipped trash cards wrap 5 -> 1.
    assert_eq!(signal.target, Some(PlayerId::new(1)));
    assert_eq!(signal.cards, [view.hands[1][0].id]);
}

#[test]
fn v3_focus_context_active_prior_play_narrowing_and_fill_in_burn() {
    let mut view = snapshot(irregular(), 3);
    let card = view.hands[1][2].id;
    view.history.push(hanabi_core::ObservedHistoryEntry {
        turn: 1,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(0),
            target: PlayerId::new(1),
            clue: Clue::Suit(Suit::new(5)),
            touched: vec![card],
            untouched: vec![],
        },
    });
    let three = connection_domain(&view, Card::new(Suit::new(5), Rank::Three));
    let five = connection_domain(&view, Card::new(Suit::new(5), Rank::Five));
    let prior = HGroupClueInterpretation {
        turn: 1,
        giver: PlayerId::new(0),
        target: PlayerId::new(1),
        clue: Clue::Rank(Rank::Three).into(),
        touched: vec![card],
        stack_heights: [0; 5],
        focus: card,
        focus_was_chop: false,
        kind: HGroupClueKind::Play,
        focus_identities: three.union(five),
        play_identities: three.union(five),
        save_identities: IdentitySet::default(),
        new_non_focus: vec![],
        non_focus_identities: vec![],
        non_focus_trash_identities: vec![],
        previously_gotten: vec![],
        hypotheses: vec![],
        unresolved_visible_prefix: vec![],
    };
    let entry = clue_entry(1, Clue::Rank(Rank::Three), vec![card]);
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let mut facts = ctx.before.facts.clone();
    ctx.before.dynamic_stack_heights = vec![1, 0];
    ctx.before.deck_size = 0;
    facts[card.index()].add_positive_clue(Clue::Rank(Rank::Three));
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    let clues = [prior];
    assert!(primary::prior_play_is_ready(
        &view,
        card,
        facts[card.index()],
        &[1, 0],
        clues.iter()
    ));
    assert!(!primary::prior_play_is_ready(
        &view,
        card,
        facts[card.index()],
        &[0, 0],
        clues.iter()
    ));
    assert_eq!(
        primary::remaining_prior_play_identities(
            &view,
            card,
            facts[card.index()],
            &[1, 0],
            clues.iter()
        ),
        three
    );
    for fill in [true, false] {
        if !fill {
            ctx.before.facts = facts.clone();
        }
        let mut effects = Effects::default();
        effects.explicit.insert(card);
        recognition::apply_stall_effects(&ctx, &view, &mut effects.borrow(&clues));
        assert!(effects.signals.has_at_turn(
            2,
            if fill {
                HGroupMoveKind::FillInClue
            } else {
                HGroupMoveKind::Burn
            }
        ));
        assert!(!effects.signals.has_at_turn(2, HGroupMoveKind::PlayClue));
        assert!(effects.pending.is_empty());
        assert!(effects.playing.is_empty());
    }
}

#[test]
fn v3_focus_context_sarcastic_precedence_requires_actor_knowledge() {
    let mut view = snapshot(irregular(), 3);
    let identity = Card::new(Suit::new(5), Rank::Three);
    advance(&mut view, identity.suit, &[Rank::One]);
    let discarded = view.hands[0][0].id;
    view.hands[1].last_mut().unwrap().identity = Some(identity);
    let sarcastic = view.hands[2][0].id;
    view.hands[2][0]
        .clues
        .record_physical(GenericClue::Color(ClueColorId::new(5)), true);
    view.hands[2][0]
        .clues
        .add_positive_clue(Clue::Rank(Rank::Three));
    view.history.push(hanabi_core::ObservedHistoryEntry {
        turn: 1,
        event: ObservedEvent::Clued {
            giver: PlayerId::new(1),
            target: PlayerId::new(0),
            clue: Clue::Rank(Rank::Three),
            touched: vec![discarded],
            untouched: vec![],
        },
    });
    let entry = hanabi_core::ObservedHistoryEntry {
        turn: 2,
        event: ObservedEvent::Discarded {
            player: PlayerId::new(0),
            card: discarded,
            identity,
        },
    };
    let mut ctx = context(&view, &entry);
    let hands = ctx.before.hands.clone();
    let facts = ctx.before.facts.clone();
    ctx.after.hands = &hands;
    ctx.after.facts = &facts;
    for known in [None, Some(identity)] {
        ctx.actor_before.discarded_identity = known;
        let mut effects = Effects::default();
        effects.explicit.insert(sarcastic);
        let borrowed = effects.borrow(&[]);
        recognition::apply_transfer_effects(
            &ctx,
            &view,
            borrowed.explicitly_clued,
            borrowed.invisibly_clued,
            borrowed.already_playing,
            borrowed.pending,
            borrowed.signals,
        );
        assert_eq!(
            effects
                .signals
                .has_at_turn(2, HGroupMoveKind::SarcasticDiscard),
            known.is_some()
        );
        assert!(
            !effects
                .signals
                .has_at_turn(2, HGroupMoveKind::GentlemansDiscard)
        );
        if known.is_some() {
            assert!(effects.invisible.contains(&sarcastic));
        } else {
            assert!(effects.pending.is_empty());
            assert!(effects.invisible.is_empty());
        }
    }
}

#[test]
fn v3_focus_context_usefulness_and_endgame_coverage_follow_custom_universe() {
    let mut view = snapshot(irregular(), 3);
    let one = Card::new(Suit::new(5), Rank::One);
    let three = Card::new(Suit::new(5), Rank::Three);
    // Four copies are physical; the standard rank-one multiplicity is three.
    view.discard_pile = (0..3).map(|index| (CardId::new(40 + index), one)).collect();
    assert!(is_eventually_useful(&view, three));
    view.discard_pile.push((CardId::new(43), one));
    assert!(!is_eventually_useful(&view, three));
    let coverage: Vec<_> = IdentitySet::all_in(view.variant.identities())
        .iter_ids()
        .map(|id| (true, IdentitySet::singleton_id(id)))
        .collect();
    assert!(primary::fully_clued_endgame(
        &view.variant,
        &[0, 0],
        0,
        3,
        coverage.iter().copied()
    ));
    assert!(!primary::fully_clued_endgame(
        &view.variant,
        &[0, 0],
        0,
        3,
        coverage[1..].iter().copied()
    ));
    // Last rank of the first (sixth-suit) stack can be covered by its stack,
    // even though the universe stores the suits and ranks in reverse order.
    let remaining: Vec<_> = coverage
        .iter()
        .copied()
        .filter(|(_, domain)| {
            *domain != connection_domain(&view, Card::new(Suit::new(5), Rank::Five))
        })
        .collect();
    assert!(primary::fully_clued_endgame(
        &view.variant,
        &[3, 0],
        0,
        3,
        remaining.into_iter()
    ));
}
