//! Shared Bluff semantics used by candidate generation and history replay.

use super::identity::{connection_card, connection_domain, connection_predecessors};
use super::{
    Card, CardId, CardSet, ConnectionManager, ConventionFacts, HGroupConnectionKind, PlayerId,
    PlayerView, Rank,
};
use hanabi_core::{ClueRank, GenericClue};

/// Literal identity established by the clue itself is safe evidence for an
/// already-clued intermediate. Do not use later clues or the clue's own
/// speculative convention claims to justify its prerequisites.
pub(super) fn literal_identity_through(view: &PlayerView, card: CardId, turn: u32) -> Option<Card> {
    let mut facts = hanabi_core::ClueFacts::default();
    for entry in view.history.iter().filter(|entry| entry.turn <= turn) {
        if let hanabi_core::ObservedEvent::Clued {
            clue,
            touched,
            untouched,
            ..
        } = &entry.event
        {
            if touched.contains(&card) {
                facts.add_positive_clue(*clue);
            } else if untouched.contains(&card) {
                facts.add_negative_clue(*clue);
            }
        }
    }
    let identities = crate::IdentitySet::from_bits(facts.identity_mask_in(&view.variant));
    (identities.len() == 1)
        .then(|| {
            identities
                .iter_ids()
                .next()
                .map(|id| connection_card(view, id))
        })
        .flatten()
}

/// A connector that is already due makes the clue a truthful continuation,
/// not a reason for the next player to invent another blind connector.
/// <https://hanabi.github.io/level-11/#bobs-truth-principle-part-1>
#[allow(clippy::too_many_arguments)]
pub(super) fn bluff_connector_is_promised(
    view: &PlayerView,
    hands: &[Vec<CardId>],
    already_playing: &CardSet,
    pending: &ConnectionManager,
    convention_facts: &ConventionFacts,
    turn: u32,
    connector: Card,
    current_focus: Option<CardId>,
) -> bool {
    hands.iter().flatten().any(|card| {
        already_playing.contains(card)
            && super::HistoricalView::new(view, turn)
                .identity(*card)
                .or_else(|| convention_facts.known_identity_before(*card, turn))
                .or_else(|| literal_identity_through(view, *card, turn))
                == Some(connector)
    }) || pending.iter().any(|connection| {
        // A Prompt selected by this clue also supplies the truthful
        // connector. Excluding every same-focus connection lets Bluff
        // recognition append an unrelated blind play to a normal Prompt.
        // Same-clue speculative Finesses remain excluded: those are the
        // interpretations whose bluff alternative is being considered.
        (Some(connection.focus) != current_focus || connection.kind == HGroupConnectionKind::Prompt)
            && connection.expected == connector
            && pending.is_active(connection)
    })
}

/// A Bluff cannot wait behind an older Finesse, but an ordinary clued play
/// does not prevent the next player from immediately demonstrating a Bluff.
/// During clue replay, exclude connections established by this very clue.
/// <https://hanabi.github.io/level-11/#queued-bluffs-illegal>
pub(super) fn bluff_is_queued(
    pending: &ConnectionManager,
    actor: PlayerId,
    current_focus: Option<CardId>,
) -> bool {
    pending.iter().any(|connection| {
        connection.actor == actor
            && connection.kind == HGroupConnectionKind::Finesse
            && Some(connection.focus) != current_focus
            && pending.is_active(connection)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BluffTargetKind {
    Ordinary,
    Three,
}

/// The missing first connector may be bluffed through already-clued higher
/// connectors. Availability must come from the caller's dated perspective.
/// <https://hanabi.github.io/level-11/#bluffs-through-already-clued-cards>
pub(super) fn bluff_through_clued_cards(
    view: &PlayerView,
    heights: &[usize],
    focus: Card,
    available: impl Fn(Card) -> bool,
) -> bool {
    let predecessors = connection_predecessors(view, heights, focus);
    !predecessors.is_empty()
        // The missing low connector must really be missing. An existing
        // clued connector makes this a truthful chain, not another Bluff.
        && (predecessors.len() == 1 || !available(predecessors[0]))
        && predecessors.iter().skip(1).all(|needed| available(*needed))
}

/// Classifies targets using only clue-time public stacks.
///
/// Ordinary Bluffs target a card one rank beyond playable. Level 13 also lets
/// a rank-3 clue target any still-useful future 3; this is the rule that makes
/// a Hard 3 Bluff possible from empty stacks.
pub(super) fn bluff_target_kind_at(
    view: &PlayerView,
    stack_heights: &[usize],
    clue: impl Into<GenericClue>,
    focus: Card,
) -> Option<BluffTargetKind> {
    let distance = connection_predecessors(view, stack_heights, focus).len();
    if distance == 1 {
        Some(BluffTargetKind::Ordinary)
    } else if clue.into() == GenericClue::Rank(ClueRank::new(3))
        && focus.rank == Rank::Three
        && distance > 1
    {
        Some(BluffTargetKind::Three)
    } else {
        None
    }
}

/// A Self-Bluff is legal with a rank clue. Suit Self-Bluffs are a separate
/// max-level family and must not be admitted by the ordinary Bluff rule.
pub(super) fn bluff_target_order_is_legal(
    clue: impl Into<GenericClue>,
    actor: PlayerId,
    target: PlayerId,
) -> bool {
    actor != target || matches!(clue.into(), GenericClue::Rank(_))
}

/// Whether the blind play connects to the clue instead of demonstrating a
/// Bluff. Color clues connect by suit. Rank clues connect only consecutive
/// ranks, so a 1 does not connect to a rank-3 clue even when both cards share
/// a suit (the Hard 3 Bluff case).
pub(super) fn bluff_play_connects(
    view: &PlayerView,
    clue: impl Into<GenericClue>,
    played: Card,
) -> bool {
    let clue = clue.into();
    match clue {
        GenericClue::Color(_) => super::physical_clue_matches(view, clue, played),
        GenericClue::Rank(_) => connection_domain(view, played)
            .iter_ids()
            .next()
            .and_then(|id| view.variant.successor(id))
            .is_some_and(|id| view.variant.touches(id, clue)),
    }
}

#[cfg(test)]
mod tests {
    use hanabi_core::{Clue, Suit};

    use super::*;

    fn view() -> PlayerView {
        hanabi_core::FullState::new_standard(2, hanabi_core::standard_deck())
            .unwrap()
            .view_for(PlayerId::new(0))
            .unwrap()
    }

    #[test]
    fn rank_bluffs_only_connect_on_consecutive_ranks() {
        assert!(!bluff_play_connects(
            &view(),
            Clue::Rank(Rank::Three),
            Card::new(Suit::Yellow, Rank::One),
        ));
        assert!(bluff_play_connects(
            &view(),
            Clue::Rank(Rank::Three),
            Card::new(Suit::Red, Rank::Two),
        ));
    }

    #[test]
    fn an_empty_stack_rank_three_is_the_special_future_target() {
        assert_eq!(
            bluff_target_kind_at(
                &view(),
                &[0; 5],
                Clue::Rank(Rank::Three),
                Card::new(Suit::Yellow, Rank::Three),
            ),
            Some(BluffTargetKind::Three),
        );
    }
}
