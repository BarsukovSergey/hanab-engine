use core::fmt;

use crate::{CardRank, SuitId};

/// Compatibility name for a physical suit identifier.
///
/// New generic code should prefer `SuitId`; this alias keeps the existing
/// No Variant API source-compatible while the rest of the engine migrates.
pub type Suit = SuitId;

/// Compatibility name for a physical card-rank identifier.
///
/// New generic code should prefer `CardRank`.
pub type Rank = CardRank;

/// The physical identity of a card.
///
/// Variant-specific clue behavior lives in `VariantSpec`, where clue colors and
/// clue ranks are intentionally separate from physical suit/rank identifiers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Card {
    pub suit: SuitId,
    pub rank: CardRank,
}

impl Card {
    #[must_use]
    pub const fn new(suit: SuitId, rank: CardRank) -> Self {
        Self { suit, rank }
    }

    /// Stable legacy index for the 25 No Variant identities.
    ///
    /// Generic variant code must use `IdentityId` instead.
    #[must_use]
    pub const fn index(self) -> usize {
        self.suit.index() * 5 + self.rank.index()
    }
}

impl fmt::Display for Card {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.suit, self.rank)
    }
}

/// Returns the 50 cards in a standard five-color deck.
///
/// The order is stable but intentionally unshuffled. A caller controls the
/// order supplied to `FullState::new_standard`, which makes tests and exact
/// world enumeration reproducible.
#[must_use]
pub fn standard_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(50);
    for suit in Suit::ALL {
        for rank in Rank::ALL {
            deck.extend(core::iter::repeat_n(
                Card::new(suit, rank),
                usize::from(rank.copies()),
            ));
        }
    }
    deck
}
