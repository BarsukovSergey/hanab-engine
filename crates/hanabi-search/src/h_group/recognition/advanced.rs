use super::{
    Card, CardId, CardSet, Clue, ConnectionManager, ConnectionObligation,
    ConnectionTransitionReason, ConventionJournal, HGroupClueInterpretation, HGroupClueKind,
    HGroupConnectionKind, HGroupMoveKind, HGroupRuleEffects, HGroupTurnContext, IdentitySet,
    ObservedEvent, ObservedHistoryEntry, PlayerView, PromiseId, Rank, RequiredFix, chop,
    finesse_position_id, five_pulled_card, next_player, protected_cards, push_signal,
    same_turn_signal, unassigned_finesse_ranks_in, was_clued_before,
};
use crate::h_group::identity::{connection_allows, connection_playable, connection_trash};
use crate::h_group::model::FixObligations;

pub(in crate::h_group) fn apply_elimination_effects(
    entry: &ObservedHistoryEntry,
    view: &PlayerView,
    hands: &[Vec<CardId>],
    stack_heights: &[usize],
    clue_tokens: u8,
    signals: &mut ConventionJournal,
) {
    // Sources: https://hanabi.github.io/level-18/#elimination--elimination-notes
    // https://hanabi.github.io/level-18/#the-elimination-blind-play
    // https://hanabi.github.io/level-18/#the-elimination-play-clue
    // https://hanabi.github.io/level-18/#the-elimination-finesse
    let historical = crate::h_group::HistoricalView::new(view, entry.turn);
    match &entry.event {
        ObservedEvent::Discarded {
            player,
            card,
            identity,
        } if clue_tokens > 0
            && !connection_trash(view, stack_heights, *identity)
            && (identity.rank == Rank::Two
                || connection_playable(view, stack_heights, *identity))
            && another_copy_survives_discard(view, entry.turn, *identity)
            && !hands.iter().enumerate().any(|(owner, hand)| {
                owner != player.index()
                    && hand.iter().any(|candidate| {
                        historical.identity(*candidate).or_else(|| {
                            signals
                                .facts()
                                .known_identity_before(*candidate, entry.turn)
                        }) == Some(*identity)
                    })
            }) =>
        {
            push_signal(
                signals,
                entry,
                *player,
                Some(*player),
                HGroupMoveKind::Elimination,
                hands[player.index()].clone(),
                Some(*identity),
            );
            let _ = card;
        }
        ObservedEvent::Clued {
            giver,
            target,
            touched,
            untouched,
            ..
        } => {
            let has_notes = signals.iter().any(|signal| {
                signal.kind == HGroupMoveKind::Elimination
                    && signal.target == Some(*target)
                    && signal.identity.is_some()
            });
            let singled_out = touched.len() == 1 || untouched.len() == 1;
            if has_notes
                && singled_out
                && touched
                    .iter()
                    .all(|card| was_clued_before(view, entry.turn, *card))
            {
                let cards = if touched.len() == 1 {
                    touched.clone()
                } else {
                    untouched.clone()
                };
                push_signal(
                    signals,
                    entry,
                    *giver,
                    Some(*target),
                    HGroupMoveKind::Elimination,
                    cards.clone(),
                    None,
                );
                push_signal(
                    signals,
                    entry,
                    *giver,
                    Some(*target),
                    if touched.iter().all(|card| {
                        historical
                            .identity(*card)
                            .is_some_and(|identity| connection_trash(view, stack_heights, identity))
                    }) {
                        HGroupMoveKind::TrashTouchElimination
                    } else {
                        HGroupMoveKind::EliminationPlayClue
                    },
                    cards,
                    None,
                );
            }
        }
        ObservedEvent::Played { .. }
        | ObservedEvent::Drew { .. }
        | ObservedEvent::Discarded { .. } => {}
    }
}

fn another_copy_survives_discard(view: &PlayerView, turn: u32, identity: Card) -> bool {
    view.history
        .iter()
        .filter(|prior| prior.turn < turn)
        .filter(|prior| {
            matches!(prior.event,
            ObservedEvent::Discarded { identity: removed, .. }
            | ObservedEvent::Played { identity: removed, successful: false, .. }
            if removed == identity)
        })
        .count()
        + 1
        < view
            .variant
            .standard_identity_id(identity)
            .and_then(|id| view.variant.copies(id))
            .map_or(0, usize::from)
}

#[allow(clippy::too_many_lines)]
pub(in crate::h_group) fn apply_elimination_resolution_effects(
    context: &HGroupTurnContext<'_>,
    view: &PlayerView,
    effects: &mut HGroupRuleEffects<'_>,
) {
    // Sources:
    // - https://hanabi.github.io/level-18/#the-elimination-blind-play
    // - https://hanabi.github.io/level-18/#the-elimination-riding-deduction
    // - https://hanabi.github.io/level-18/#the-elimination-self-chop-move
    let entry = context.entry;
    let claims = effects
        .signals
        .facts()
        .identity_claims()
        .iter()
        .filter(|claim| claim.target.is_some())
        .filter(|claim| {
            matches!(
                claim.source,
                HGroupMoveKind::Elimination | HGroupMoveKind::EliminationRewrite
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    for claim in claims {
        let target = claim
            .target
            .expect("only targeted Elimination claims are selected");
        let candidates_after =
            remaining_elimination_candidates(context, view, target, &claim.cards, claim.identity);
        // Elimination is an inference about the remaining hand, not a rule
        // that can overrule later literal clues. If every original slot has
        // left the hand or been explicitly ruled out, retire the claim rather
        // than passing an impossible existential constraint to world building.
        if candidates_after.is_empty() {
            push_signal(
                effects.signals,
                entry,
                claim.actor,
                Some(target),
                HGroupMoveKind::Retraction,
                claim.cards.clone(),
                Some(claim.identity),
            );
            continue;
        }
        if connection_trash(view, &context.after.dynamic_stack_heights, claim.identity) {
            for card in &claim.cards {
                effects.forced_playable.remove(card);
                effects.chop_moved.remove(card);
            }
            continue;
        }
        if candidates_after.len() == 1
            && !effects.explicitly_clued.contains(&candidates_after[0])
            && !effects.signals.iter().any(|signal| {
                signal.turn < entry.turn
                    && signal.kind == HGroupMoveKind::EliminationSelfChopMove
                    && signal.cards == candidates_after
            })
        {
            let card = candidates_after[0];
            effects.chop_moved.insert(card);
            if connection_playable(view, &context.after.dynamic_stack_heights, claim.identity) {
                effects.forced_playable.insert(card);
            }
            push_signal(
                effects.signals,
                entry,
                claim.actor,
                Some(target),
                HGroupMoveKind::EliminationSelfChopMove,
                vec![card],
                None,
            );
        }

        let ObservedEvent::Played {
            player,
            card,
            successful: true,
            ..
        } = entry.event
        else {
            continue;
        };
        if player != target || !claim.cards.contains(&card) {
            continue;
        }
        let candidates_before = compatible_elimination_candidates(
            view,
            &context.before.hands[target.index()],
            &context.before.facts,
            &claim.cards,
            claim.identity,
        );
        let kind = if candidates_before.len() == 1 {
            Some(HGroupMoveKind::EliminationBlindPlay)
        } else if candidates_before.len() == 2
            && chop(&context.before.hands[target.index()], effects.chop_moved) != Some(card)
        {
            Some(HGroupMoveKind::EliminationRiding)
        } else {
            None
        };
        if let Some(kind) = kind {
            push_signal(
                effects.signals,
                entry,
                player,
                Some(target),
                kind,
                vec![card],
                None,
            );
        }
    }
}

fn remaining_elimination_candidates(
    context: &HGroupTurnContext<'_>,
    view: &PlayerView,
    target: hanabi_core::PlayerId,
    cards: &[CardId],
    identity: Card,
) -> Vec<CardId> {
    if known_copies_outside_hand(context, view, identity, target)
        >= view
            .variant
            .standard_identity_id(identity)
            .and_then(|id| view.variant.copies(id))
            .map_or(0, usize::from)
    {
        return Vec::new();
    }
    compatible_elimination_candidates(
        view,
        &context.after.hands[target.index()],
        context.after.facts,
        cards,
        identity,
    )
}

fn compatible_elimination_candidates(
    view: &PlayerView,
    hand: &[CardId],
    facts: &[hanabi_core::ClueFacts],
    cards: &[CardId],
    identity: Card,
) -> Vec<CardId> {
    hand.iter()
        .copied()
        .filter(|card| {
            cards.contains(card) && connection_allows(view, facts[card.index()], identity)
        })
        .collect()
}

fn known_copies_outside_hand(
    context: &HGroupTurnContext<'_>,
    view: &PlayerView,
    identity: Card,
    target: hanabi_core::PlayerId,
) -> usize {
    let removed = view.history.iter().filter(|entry| entry.turn <= context.entry.turn)
        .filter(|entry| matches!(entry.event,
            ObservedEvent::Played { identity: card, .. } | ObservedEvent::Discarded { identity: card, .. } if card == identity
        )).count();
    let historical = crate::h_group::HistoricalView::new(view, context.entry.turn + 1);
    removed
        + context
            .after
            .hands
            .iter()
            .enumerate()
            .filter(|(owner, _)| *owner != target.index())
            .flat_map(|(_, hand)| hand.iter())
            .filter(|card| historical.identity(**card) == Some(identity))
            .count()
}
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(in crate::h_group) fn apply_five_tech_effects(
    context: &HGroupTurnContext<'_>,
    view: &PlayerView,
    hands: &[Vec<CardId>],
    clues: &[HGroupClueInterpretation],
    stack_heights: &[usize],
    explicitly_clued: &CardSet,
    invisibly_clued: &CardSet,
    chop_moved: &mut CardSet,
    pending: &mut ConnectionManager,
    forced_playable: &mut CardSet,
    implicit_saves: &mut Vec<(CardId, IdentitySet)>,
    signals: &mut ConventionJournal,
) {
    // Sources: https://hanabi.github.io/level-19/#the-early-5s-chop-move
    // https://hanabi.github.io/level-19/#the-5-pull
    // https://hanabi.github.io/level-19/#the-5-number-ejection-5ne
    // https://hanabi.github.io/level-19/#the-5-number-discharge-5nd
    let entry = context.entry;
    let historical = context.historical;
    let ObservedEvent::Clued {
        giver,
        target,
        clue: Clue::Rank(Rank::Five),
        touched,
        ..
    } = &entry.event
    else {
        return;
    };
    if touched.is_empty() {
        return;
    }
    let repeated_five = touched
        .iter()
        .all(|card| was_clued_before(view, entry.turn, *card));
    let gotten = protected_cards(explicitly_clued, invisibly_clued, chop_moved);
    let target_chop = chop(&hands[target.index()], &gotten);
    let visible_saved_two = clues
        .iter()
        .rev()
        .filter(|interpretation| interpretation.turn < entry.turn)
        .filter(|interpretation| interpretation.target != *target)
        .filter(|interpretation| !interpretation.save_identities.is_empty())
        .find_map(|interpretation| {
            let twos = IdentitySet::from_bits(
                interpretation
                    .save_identities
                    .iter_in(view.variant.identities())
                    .filter(|identity| identity.rank == Rank::Two)
                    .fold(0, |mask, identity| {
                        mask | crate::h_group::identity::connection_bit(view, identity)
                    }),
            );
            (!twos.is_empty()).then_some(twos)
        });
    if repeated_five
        && same_turn_signal(signals, entry.turn, HGroupMoveKind::FiveStall)
        && same_turn_signal(signals, entry.turn, HGroupMoveKind::Stall)
    {
        if let (Some(chop), Some(identities)) = (target_chop, visible_saved_two) {
            // Interaction Between 2 Saves & 5 Stalls: the receiver excludes
            // their chop (the giver could have saved it directly) and writes
            // elimination notes on every otherwise-unclued off-chop card.
            // Source: https://hanabi.github.io/level-19/#interaction-between-2-saves--5-stalls
            let candidates = hands[target.index()]
                .iter()
                .filter(|card| {
                    **card != chop
                        && !touched.contains(card)
                        && !explicitly_clued.contains(card)
                        && identities
                            .iter_in(view.variant.identities())
                            .any(|identity| {
                                connection_allows(
                                    view,
                                    context.before.facts[card.index()],
                                    identity,
                                )
                            })
                })
                .copied()
                .collect::<Vec<_>>();
            if !candidates.is_empty() {
                implicit_saves.extend(candidates.iter().copied().map(|card| (card, identities)));
                push_signal(
                    signals,
                    entry,
                    *giver,
                    Some(*target),
                    HGroupMoveKind::FiveStall,
                    touched.clone(),
                    None,
                );
                return;
            }
        }
    }
    if signals
        .iter()
        .any(|signal| signal.turn == entry.turn && signal.kind == HGroupMoveKind::FiveStall)
    {
        return;
    }
    // Save clues take precedence over 5 tech. A rank-5 clue that can be a
    // 5 Save cannot simultaneously pull the adjacent card.
    if clues.last().is_some_and(|interpretation| {
        interpretation.turn == entry.turn && !interpretation.save_identities.is_empty()
    }) {
        return;
    }
    let Some(pulled) = five_pulled_card(&hands[target.index()], touched, &gotten) else {
        return;
    };
    let Some(identity) = historical.identity(pulled) else {
        return;
    };
    let predecessors =
        crate::h_group::identity::connection_predecessors(view, stack_heights, identity);
    let actor = next_player(*giver, hands.len());
    let (kind, forced) = if connection_trash(view, stack_heights, identity) {
        let Some(card) = finesse_position_id(&hands[actor.index()], &gotten, 2).filter(|card| {
            historical
                .identity(*card)
                .is_some_and(|candidate| connection_playable(view, stack_heights, candidate))
        }) else {
            return;
        };
        chop_moved.insert(pulled);
        (HGroupMoveKind::FiveNumberDischarge, Some(card))
    } else if connection_playable(view, stack_heights, identity) {
        (HGroupMoveKind::FivePull, Some(pulled))
    } else if predecessors.len() == 1 {
        if actor == *target {
            return;
        }
        let connector = predecessors[0];
        let Some(card) = finesse_position_id(&hands[actor.index()], &gotten, 0)
            .filter(|card| historical.identity(*card) == Some(connector))
        else {
            return;
        };
        pending.cancel_where(
            entry.turn,
            ConnectionTransitionReason::Superseded,
            |connection| connection.actor == actor || connection.actor == *target,
        );
        pending.start(
            entry.turn,
            ConnectionObligation {
                promise: PromiseId::UNASSIGNED,
                actor,
                cards: vec![card],
                expected: connector,
                focus_identity: identity,
                kind: HGroupConnectionKind::Finesse,
                focus: pulled,
                step: 0,
            },
        );
        pending.start(
            entry.turn,
            ConnectionObligation {
                promise: PromiseId::UNASSIGNED,
                actor: *target,
                cards: vec![pulled],
                expected: identity,
                focus_identity: identity,
                kind: HGroupConnectionKind::Finesse,
                focus: pulled,
                step: 1,
            },
        );
        (HGroupMoveKind::FivePull, None)
    } else {
        let Some(card) = finesse_position_id(&hands[actor.index()], &gotten, 1).filter(|card| {
            historical
                .identity(*card)
                .is_some_and(|candidate| connection_playable(view, stack_heights, candidate))
        }) else {
            return;
        };
        chop_moved.insert(pulled);
        (HGroupMoveKind::FiveNumberEjection, Some(card))
    };
    if let Some(forced) = forced {
        pending.cancel_where(
            entry.turn,
            ConnectionTransitionReason::Superseded,
            |connection| connection.actor == actor,
        );
        forced_playable.insert(forced);
    }
    push_signal(
        signals,
        entry,
        *giver,
        Some(*target),
        kind,
        touched.clone(),
        touched.iter().find_map(|card| historical.identity(*card)),
    );
    if matches!(
        kind,
        HGroupMoveKind::FiveNumberEjection | HGroupMoveKind::FiveNumberDischarge
    ) {
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            if kind == HGroupMoveKind::FiveNumberEjection {
                HGroupMoveKind::Ejection
            } else {
                HGroupMoveKind::Discharge
            },
            touched.clone(),
            touched.iter().find_map(|card| historical.identity(*card)),
        );
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(in crate::h_group) fn apply_out_of_order_effects(
    entry: &ObservedHistoryEntry,
    view: &PlayerView,
    hands: &[Vec<CardId>],
    clues: &[HGroupClueInterpretation],
    stack_heights: &[usize],
    explicitly_clued: &CardSet,
    invisibly_clued: &CardSet,
    chop_moved: &CardSet,
    pending: &mut ConnectionManager,
    _forced_playable: &mut CardSet,
    required_fixes: &mut FixObligations,
    signals: &mut ConventionJournal,
) {
    // Sources: https://hanabi.github.io/level-20/#the-occupied-play-clue--the-occupied-finesse-opc
    // https://hanabi.github.io/level-20/#the-out-of-order-play-clue-triple-o--ooo
    // https://hanabi.github.io/level-20/#the-out-of-order-finesse
    // https://hanabi.github.io/level-20/#the-suboptimal-prompt--the-suboptimal-finesse--the-suboptimal-bluff
    // https://hanabi.github.io/level-20/#the-no-information-finesse
    // https://hanabi.github.io/level-20/#the-no-information-double-bluff-nidb
    let historical = crate::h_group::HistoricalView::new(view, entry.turn);
    let ObservedEvent::Clued {
        giver,
        target,
        touched,
        ..
    } = &entry.event
    else {
        return;
    };
    let interpretation = clues
        .iter()
        .rev()
        .find(|clue| clue.turn == entry.turn)
        .filter(|clue| {
            matches!(clue.kind, HGroupClueKind::Play)
                || (clue.kind == HGroupClueKind::Unrecognized && clue.save_identities.is_empty())
        });
    let four_charm = interpretation.is_some_and(|meaning| {
        if meaning.hypotheses.iter().any(|hypothesis| {
            hypothesis.connection_steps.iter().any(|step| {
                step.kind == HGroupConnectionKind::Prompt
                    && step
                        .cards
                        .iter()
                        .any(|card| was_clued_before(view, entry.turn, *card))
            })
        }) {
            return false;
        }
        if *target == next_player(*giver, hands.len())
            || was_clued_before(view, entry.turn, meaning.focus)
        {
            return false;
        }
        let Some(focus_identity) = historical.identity(meaning.focus) else {
            return false;
        };
        if focus_identity.rank != Rank::Four
            || !view
                .variant
                .stack_suits()
                .iter()
                .position(|suit| *suit == focus_identity.suit)
                .is_some_and(|suit| stack_heights[suit] == 0)
        {
            return false;
        }
        if same_turn_signal(signals, entry.turn, HGroupMoveKind::CriticalColorBluff)
            || same_turn_signal(signals, entry.turn, HGroupMoveKind::DoubleBluff)
        {
            return false;
        }
        let actor = next_player(*giver, hands.len());
        if unassigned_finesse_ranks_in(
            view,
            *giver,
            actor,
            focus_identity,
            stack_heights,
            explicitly_clued,
            entry.turn,
        ) < 3
        {
            return false;
        }
        let gotten = protected_cards(explicitly_clued, invisibly_clued, chop_moved);
        let charmed = finesse_position_id(&hands[actor.index()], &gotten, 3).is_some_and(|card| {
            historical
                .identity(card)
                .map_or(actor == view.observer, |identity| {
                    connection_playable(view, stack_heights, identity)
                })
        });
        let double_bluff_actor = next_player(actor, hands.len());
        let double_bluff_available =
            finesse_position_id(&hands[double_bluff_actor.index()], &gotten, 0)
                .and_then(|card| historical.identity(card))
                .is_some_and(|identity| connection_playable(view, stack_heights, identity));
        charmed && !double_bluff_available
    });
    if four_charm {
        // A 4 Charm replaces the apparent multi-card Out-of-Order Play. It
        // must not manufacture a Fix obligation for the focused 4.
        // Source: https://hanabi.github.io/level-23/#the-4-charm
        return;
    }
    if let Some(card) = interpretation.map(|clue| clue.focus).filter(|card| {
        historical.identity(*card).is_some_and(|identity| {
            let predecessors =
                crate::h_group::identity::connection_predecessors(view, stack_heights, identity);
            !predecessors.is_empty()
                && touched.iter().any(|candidate| {
                    candidate != card
                        && interpretation
                            .is_some_and(|clue| !clue.previously_gotten.contains(candidate))
                        && historical
                            .identity(*candidate)
                            .is_some_and(|lower| predecessors.contains(&lower))
                })
        })
    }) {
        let focus = card;
        let focus_identity = historical.identity(focus);
        if let Some(identity) = focus_identity {
            required_fixes.insert_unconditional(RequiredFix {
                actor: next_player(*giver, view.hands.len()),
                target: *target,
                focus,
                identity,
            });
        }
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            HGroupMoveKind::OccupiedPlay,
            vec![focus],
            focus_identity,
        );
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            if pending.iter().any(|connection| connection.focus == focus) {
                HGroupMoveKind::OutOfOrderFinesse
            } else {
                HGroupMoveKind::OutOfOrderPlay
            },
            vec![focus],
            focus_identity,
        );
    }
    let current_connections = pending
        .iter()
        .filter(|connection| {
            interpretation.is_some_and(|meaning| connection.focus == meaning.focus)
        })
        .collect::<Vec<_>>();
    let no_information = !touched.is_empty()
        && touched
            .iter()
            .all(|card| was_clued_before(view, entry.turn, *card));
    let suboptimal = current_connections.iter().any(|connection| {
        connection.cards.len() > 1
            || connection
                .cards
                .first()
                .is_some_and(|card| hands[connection.actor.index()].last() != Some(card))
    });
    if suboptimal {
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            HGroupMoveKind::SuboptimalConnection,
            current_connections
                .iter()
                .flat_map(|connection| connection.cards.iter().copied())
                .collect(),
            None,
        );
    }
    if no_information && !current_connections.is_empty() {
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            HGroupMoveKind::NoInformationFinesse,
            current_connections
                .iter()
                .flat_map(|connection| connection.cards.iter().copied())
                .collect(),
            None,
        );
    }
    if no_information && same_turn_signal(signals, entry.turn, HGroupMoveKind::DoubleBluff) {
        push_signal(
            signals,
            entry,
            *giver,
            Some(*target),
            HGroupMoveKind::NoInformationDoubleBluff,
            touched.clone(),
            None,
        );
    }
}
