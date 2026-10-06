use hanabi_core::{Card, CardId, ObservedEvent, PlayerView};

use super::{CardSet, ConventionFacts, HGroupCardInference};

/// Adapters for retained Card-valued promises. Masks always use the active universe.
pub(super) fn connection_domain(view: &PlayerView, card: Card) -> crate::IdentitySet {
    view.variant.standard_identity_id(card).map_or_else(
        crate::IdentitySet::default,
        crate::IdentitySet::singleton_id,
    )
}

pub(super) fn connection_bit(view: &PlayerView, card: Card) -> u64 {
    view.variant
        .standard_identity_id(card)
        .map_or(0, |id| 1_u64 << id.index())
}

/// The retained convention-fact exclusion store uses Card positions. Translate
/// at the consumer boundary; it must never be combined with active-ID masks.
pub(super) fn connection_fact_domain(
    view: &PlayerView,
    legacy: crate::IdentitySet,
) -> crate::IdentitySet {
    crate::IdentitySet::from_bits(
        view.variant
            .identities()
            .identities()
            .iter()
            .filter(|identity| legacy.contains(Card::new(identity.suit, identity.rank)))
            .fold(0, |mask, identity| mask | (1_u64 << identity.id.index())),
    )
}

pub(super) fn connection_allows(
    view: &PlayerView,
    facts: hanabi_core::ClueFacts,
    card: Card,
) -> bool {
    view.variant
        .standard_identity_id(card)
        .is_some_and(|id| facts.identity_mask_in(&view.variant) & (1 << id.index()) != 0)
}

pub(super) fn connection_card(view: &PlayerView, id: hanabi_core::IdentityId) -> Card {
    let identity = view
        .variant
        .identities()
        .get(id)
        .expect("active connection identity");
    Card::new(identity.suit, identity.rank)
}

pub(super) fn connection_playable(view: &PlayerView, heights: &[usize], card: Card) -> bool {
    view.variant
        .standard_identity_id(card)
        .is_some_and(|id| view.variant.playable(heights, id))
}

pub(super) fn connection_trash(view: &PlayerView, heights: &[usize], card: Card) -> bool {
    !view
        .variant
        .standard_identity_id(card)
        .is_some_and(|id| connection_live(view, heights, id))
}

pub(super) fn connection_successor(view: &PlayerView, card: Card) -> Option<Card> {
    view.variant
        .standard_identity_id(card)
        .and_then(|id| view.variant.successor(id))
        .map(|id| connection_card(view, id))
}

pub(super) fn connection_predecessor(view: &PlayerView, card: Card) -> Option<Card> {
    view.variant
        .standard_identity_id(card)
        .and_then(|id| view.variant.predecessor(id))
        .map(|id| connection_card(view, id))
}

pub(super) fn connection_live(
    view: &PlayerView,
    heights: &[usize],
    id: hanabi_core::IdentityId,
) -> bool {
    let Some(identity) = view.variant.identities().get(id) else {
        return false;
    };
    let Some(suit) = view
        .variant
        .stack_suits()
        .iter()
        .position(|suit| *suit == identity.suit)
    else {
        return false;
    };
    let Some(rank) = view
        .variant
        .card_ranks()
        .iter()
        .position(|rank| *rank == identity.rank)
    else {
        return false;
    };
    heights.get(suit).is_some_and(|height| rank >= *height)
}

pub(super) fn connection_advance(view: &PlayerView, heights: &mut [usize], card: Card) {
    let suit = view
        .variant
        .stack_suits()
        .iter()
        .position(|suit| *suit == card.suit)
        .expect("active stack");
    let rank = view
        .variant
        .card_ranks()
        .iter()
        .position(|rank| *rank == card.rank)
        .expect("active rank");
    heights[suit] = rank + 1;
}

pub(super) fn connection_predecessors(
    view: &PlayerView,
    heights: &[usize],
    card: Card,
) -> Vec<Card> {
    let Some(mut id) = view.variant.standard_identity_id(card) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    while let Some(previous) = view.variant.predecessor(id) {
        let identity = connection_card(view, previous);
        let suit = view
            .variant
            .stack_suits()
            .iter()
            .position(|suit| *suit == identity.suit)
            .expect("active stack");
        let rank = view
            .variant
            .card_ranks()
            .iter()
            .position(|rank| *rank == identity.rank)
            .expect("active rank");
        if rank < heights[suit] {
            break;
        }
        result.push(identity);
        id = previous;
    }
    result.reverse();
    result
}

/// Public stack state immediately before a historical event, or at the current view.
pub(super) fn connection_heights(view: &PlayerView, turn: Option<u32>) -> Vec<usize> {
    let Some(turn) = turn else {
        return view.play_stacks.iter().map(Vec::len).collect();
    };
    let mut heights = vec![0; view.variant.stack_suits().len()];
    for entry in view.history.iter().take_while(|entry| entry.turn < turn) {
        if let ObservedEvent::Played {
            identity,
            successful: true,
            ..
        } = entry.event
        {
            connection_advance(view, &mut heights, identity);
        }
    }
    heights
}

/// Identity visible in the source view at its current turn. Historical rule
/// interpretation must use `HistoricalView` instead.
pub(super) fn identity_of<C>(view: &PlayerView<C>, card: CardId) -> Option<Card> {
    view.hands
        .iter()
        .flatten()
        .find(|candidate| candidate.id == card)
        .and_then(|candidate| candidate.identity)
        .or_else(|| {
            view.play_stacks
                .iter()
                .flatten()
                .chain(view.discard_pile.iter())
                .find_map(|(candidate, identity)| (*candidate == card).then_some(*identity))
        })
        .or_else(|| {
            view.history.iter().find_map(|entry| match entry.event {
                ObservedEvent::Played {
                    card: candidate,
                    identity,
                    ..
                }
                | ObservedEvent::Discarded {
                    card: candidate,
                    identity,
                    ..
                } if candidate == card => Some(identity),
                ObservedEvent::Drew {
                    card: candidate,
                    identity: Some(identity),
                    ..
                } if candidate == card => Some(identity),
                _ => None,
            })
        })
}

pub(super) fn is_playable_now(view: &PlayerView, identity: Card) -> bool {
    connection_playable(view, &connection_heights(view, None), identity)
}

pub(super) fn is_playable_at(stack_heights: [u8; 5], identity: Card) -> bool {
    identity.rank.number() == stack_heights[identity.suit.index()] + 1
}

pub(super) fn is_trash_at(stack_heights: [u8; 5], identity: Card) -> bool {
    identity.rank.number() <= stack_heights[identity.suit.index()]
}

pub(super) fn card_is_trash(view: &PlayerView, identity: Card) -> bool {
    !view
        .variant
        .standard_identity_id(identity)
        .is_some_and(|id| connection_live(view, &connection_heights(view, None), id))
}

pub(super) fn is_eventually_useful(view: &PlayerView, identity: Card) -> bool {
    if card_is_trash(view, identity) {
        return false;
    }
    connection_predecessors(view, &connection_heights(view, None), identity)
        .into_iter()
        .all(|lower| {
            view.discard_pile
                .iter()
                .filter(|(_, card)| *card == lower)
                .count()
                < usize::from(
                    view.variant
                        .copies(
                            view.variant
                                .standard_identity_id(lower)
                                .expect("active predecessor"),
                        )
                        .expect("active copies"),
                )
        })
}

/// Duplicate accounting requires clued or promised copies, not merely protected
/// ones. A Chop-Moved copy still needs information before its owner can play it.
/// <https://hanabi.github.io/beginner/good-touch-principle>
pub(super) fn is_convention_trash(
    view: &PlayerView,
    identity: Card,
    clued_or_promised: &CardSet,
    own_notes: &[HGroupCardInference],
) -> bool {
    if !is_eventually_useful(view, identity) {
        return true;
    }
    view.hands
        .iter()
        .flatten()
        .filter(|card| clued_or_promised.contains(&card.id))
        .filter(|card| {
            card.identity == Some(identity)
                || (card.identity.is_none()
                    && own_notes.iter().any(|note| {
                        note.card == card.id
                            && note.identities.len() == 1
                            && note
                                .identities
                                .intersection(connection_domain(view, identity))
                                .len()
                                == 1
                    }))
        })
        .take(2)
        .count()
        >= 2
}

/// Whether this specific card is useless because its identity is either
/// already played or already represented by another protected card.
///
/// Unlike [`is_convention_trash`], this form is suitable while compiling a
/// clue event: the newly touched card may still have several possible
/// identities, and each possibility must be checked against exact convention
/// facts established elsewhere on the board.
pub(super) fn is_card_identity_accounted_trash(
    view: &PlayerView,
    historical: super::HistoricalView<'_>,
    card: CardId,
    identity: Card,
    stack_heights: &[usize],
    gotten: &CardSet,
    convention_facts: &ConventionFacts,
) -> bool {
    !view
        .variant
        .standard_identity_id(identity)
        .is_some_and(|id| connection_live(view, stack_heights, id))
        || gotten.iter().copied().any(|other| {
            other != card
                && (historical.identity(other) == Some(identity)
                    || convention_facts.known_identity(other) == Some(identity))
        })
}
