use core::fmt;

use crate::{Card, Rank, Suit};

pub const MAX_IDENTITIES: usize = 64;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u8);

        impl $name {
            #[must_use]
            pub const fn new(value: u8) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn value(self) -> u8 {
                self.0
            }

        }
    };
}

id_type!(SuitId);
id_type!(ClueColorId);
id_type!(CardRank);
id_type!(ClueRank);
id_type!(IdentityId);

impl IdentityId {
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(self.value())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GenericClue {
    Color(ClueColorId),
    Rank(ClueRank),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentitySpec {
    pub suit: SuitId,
    pub rank: CardRank,
    pub copies: u8,
    pub clue_colors: Vec<ClueColorId>,
    pub clue_ranks: Vec<ClueRank>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identity {
    pub id: IdentityId,
    pub suit: SuitId,
    pub rank: CardRank,
    pub copies: u8,
    pub clue_colors: Vec<ClueColorId>,
    pub clue_ranks: Vec<ClueRank>,
}

impl Identity {
    #[must_use]
    pub fn touched_by(&self, clue: GenericClue) -> bool {
        match clue {
            GenericClue::Color(color) => self.clue_colors.contains(&color),
            GenericClue::Rank(rank) => self.clue_ranks.contains(&rank),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityUniverse {
    identities: Vec<Identity>,
    deck_size: usize,
}

impl IdentityUniverse {
    fn new(identities: Vec<IdentitySpec>) -> Result<Self, VariantSpecError> {
        if identities.is_empty() {
            return Err(VariantSpecError::NoIdentities);
        }
        if identities.len() > MAX_IDENTITIES {
            return Err(VariantSpecError::TooManyIdentities(identities.len()));
        }

        let mut built = Vec::with_capacity(identities.len());
        let mut deck_size = 0_usize;
        for (index, identity) in identities.into_iter().enumerate() {
            if identity.copies == 0 {
                return Err(VariantSpecError::ZeroCopies {
                    identity_index: index,
                });
            }
            if built.iter().any(|existing: &Identity| {
                existing.suit == identity.suit && existing.rank == identity.rank
            }) {
                return Err(VariantSpecError::DuplicateIdentity {
                    suit: identity.suit,
                    rank: identity.rank,
                });
            }
            deck_size = deck_size
                .checked_add(usize::from(identity.copies))
                .ok_or(VariantSpecError::DeckTooLarge)?;
            built.push(Identity {
                id: IdentityId::new(u8::try_from(index).expect("at most 64 identities")),
                suit: identity.suit,
                rank: identity.rank,
                copies: identity.copies,
                clue_colors: identity.clue_colors,
                clue_ranks: identity.clue_ranks,
            });
        }
        Ok(Self {
            identities: built,
            deck_size,
        })
    }

    #[must_use]
    pub fn identities(&self) -> &[Identity] {
        &self.identities
    }

    #[must_use]
    pub fn get(&self, id: IdentityId) -> Option<&Identity> {
        self.identities.get(id.index())
    }

    #[must_use]
    pub fn id_for(&self, suit: SuitId, rank: CardRank) -> Option<IdentityId> {
        self.identities
            .iter()
            .find(|identity| identity.suit == suit && identity.rank == rank)
            .map(|identity| identity.id)
    }

    #[must_use]
    pub const fn deck_size(&self) -> usize {
        self.deck_size
    }

    #[must_use]
    pub fn all_mask(&self) -> u64 {
        if self.identities.len() == MAX_IDENTITIES {
            u64::MAX
        } else {
            (1_u64 << self.identities.len()) - 1
        }
    }

    #[must_use]
    pub fn clue_mask(&self, clue: GenericClue) -> u64 {
        self.identities.iter().fold(0_u64, |mask, identity| {
            if identity.touched_by(clue) {
                mask | (1_u64 << identity.id.index())
            } else {
                mask
            }
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantSpec {
    name: String,
    stack_suits: Vec<SuitId>,
    card_ranks: Vec<CardRank>,
    clue_colors: Vec<ClueColorId>,
    clue_ranks: Vec<ClueRank>,
    universe: IdentityUniverse,
}

impl VariantSpec {
    pub fn new(
        name: impl Into<String>,
        stack_suits: Vec<SuitId>,
        card_ranks: Vec<CardRank>,
        clue_colors: Vec<ClueColorId>,
        clue_ranks: Vec<ClueRank>,
        identities: Vec<IdentitySpec>,
    ) -> Result<Self, VariantSpecError> {
        if stack_suits.is_empty() {
            return Err(VariantSpecError::NoStackSuits);
        }
        ensure_unique("stack suit", &stack_suits)?;
        ensure_unique("card rank", &card_ranks)?;
        ensure_unique("clue color", &clue_colors)?;
        ensure_unique("clue rank", &clue_ranks)?;
        if card_ranks.is_empty() {
            return Err(VariantSpecError::NoCardRanks);
        }

        for (index, identity) in identities.iter().enumerate() {
            if !stack_suits.contains(&identity.suit) {
                return Err(VariantSpecError::UnknownIdentitySuit {
                    identity_index: index,
                    suit: identity.suit,
                });
            }
            if !card_ranks.contains(&identity.rank) {
                return Err(VariantSpecError::UnknownIdentityCardRank {
                    identity_index: index,
                    rank: identity.rank,
                });
            }
            for color in &identity.clue_colors {
                if !clue_colors.contains(color) {
                    return Err(VariantSpecError::UnknownIdentityColor {
                        identity_index: index,
                        color: *color,
                    });
                }
            }
            for rank in &identity.clue_ranks {
                if !clue_ranks.contains(rank) {
                    return Err(VariantSpecError::UnknownIdentityClueRank {
                        identity_index: index,
                        rank: *rank,
                    });
                }
            }
        }

        Ok(Self {
            name: name.into(),
            stack_suits,
            card_ranks,
            clue_colors,
            clue_ranks,
            universe: IdentityUniverse::new(identities)?,
        })
    }

    #[must_use]
    pub fn no_variant() -> Self {
        let stack_suits = (0_u8..5).map(SuitId::new).collect::<Vec<_>>();
        let card_ranks = (1_u8..=5).map(CardRank::new).collect::<Vec<_>>();
        let clue_colors = (0_u8..5).map(ClueColorId::new).collect::<Vec<_>>();
        let clue_ranks = (1_u8..=5).map(ClueRank::new).collect::<Vec<_>>();
        let mut identities = Vec::with_capacity(25);
        for suit in 0_u8..5 {
            for rank in 1_u8..=5 {
                let copies = match rank {
                    1 => 3,
                    2..=4 => 2,
                    5 => 1,
                    _ => unreachable!(),
                };
                identities.push(IdentitySpec {
                    suit: SuitId::new(suit),
                    rank: CardRank::new(rank),
                    copies,
                    clue_colors: vec![ClueColorId::new(suit)],
                    clue_ranks: vec![ClueRank::new(rank)],
                });
            }
        }
        Self::new(
            "No Variant",
            stack_suits,
            card_ranks,
            clue_colors,
            clue_ranks,
            identities,
        )
        .expect("built-in No Variant spec is valid")
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn stack_suits(&self) -> &[SuitId] {
        &self.stack_suits
    }

    #[must_use]
    pub fn card_ranks(&self) -> &[CardRank] {
        &self.card_ranks
    }

    #[must_use]
    pub fn clue_colors(&self) -> &[ClueColorId] {
        &self.clue_colors
    }

    #[must_use]
    pub fn clue_ranks(&self) -> &[ClueRank] {
        &self.clue_ranks
    }

    #[must_use]
    pub const fn identities(&self) -> &IdentityUniverse {
        &self.universe
    }

    #[must_use]
    pub fn touches(&self, identity: IdentityId, clue: GenericClue) -> bool {
        self.universe
            .get(identity)
            .is_some_and(|identity| identity.touched_by(clue))
    }

    #[must_use]
    pub fn copies(&self, identity: IdentityId) -> Option<u8> {
        self.universe.get(identity).map(|identity| identity.copies)
    }

    #[must_use]
    pub fn legal_clues(&self, target_hand: &[IdentityId]) -> Vec<GenericClue> {
        let colors = self.clue_colors.iter().copied().map(GenericClue::Color);
        let ranks = self.clue_ranks.iter().copied().map(GenericClue::Rank);
        colors
            .chain(ranks)
            .filter(|clue| {
                target_hand
                    .iter()
                    .any(|identity| self.touches(*identity, *clue))
            })
            .collect()
    }

    #[must_use]
    pub fn successor(&self, identity: IdentityId) -> Option<IdentityId> {
        let identity = self.universe.get(identity)?;
        let position = self
            .card_ranks
            .iter()
            .position(|rank| *rank == identity.rank)?;
        let next_rank = *self.card_ranks.get(position + 1)?;
        self.universe.id_for(identity.suit, next_rank)
    }

    #[must_use]
    pub fn playable(&self, stack_heights: &[usize], identity: IdentityId) -> bool {
        let Some(identity) = self.universe.get(identity) else {
            return false;
        };
        let Some(suit_index) = self
            .stack_suits
            .iter()
            .position(|suit| *suit == identity.suit)
        else {
            return false;
        };
        let Some(rank_index) = self
            .card_ranks
            .iter()
            .position(|rank| *rank == identity.rank)
        else {
            return false;
        };
        stack_heights.get(suit_index).copied() == Some(rank_index)
    }

    #[must_use]
    pub fn max_score(&self) -> usize {
        self.stack_suits
            .iter()
            .map(|suit| {
                self.card_ranks
                    .iter()
                    .take_while(|rank| self.universe.id_for(*suit, **rank).is_some())
                    .count()
            })
            .sum()
    }
}

impl From<Suit> for SuitId {
    fn from(value: Suit) -> Self {
        Self::new(u8::try_from(value.index()).expect("five standard suits"))
    }
}

impl From<Rank> for CardRank {
    fn from(value: Rank) -> Self {
        Self::new(value.number())
    }
}

impl From<Rank> for ClueRank {
    fn from(value: Rank) -> Self {
        Self::new(value.number())
    }
}

impl VariantSpec {
    #[must_use]
    pub fn standard_identity_id(&self, card: Card) -> Option<IdentityId> {
        self.universe
            .id_for(SuitId::from(card.suit), CardRank::from(card.rank))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantSpecError {
    NoStackSuits,
    NoCardRanks,
    NoIdentities,
    TooManyIdentities(usize),
    ZeroCopies {
        identity_index: usize,
    },
    DeckTooLarge,
    DuplicateValue {
        kind: &'static str,
        value: u8,
    },
    DuplicateIdentity {
        suit: SuitId,
        rank: CardRank,
    },
    UnknownIdentitySuit {
        identity_index: usize,
        suit: SuitId,
    },
    UnknownIdentityCardRank {
        identity_index: usize,
        rank: CardRank,
    },
    UnknownIdentityColor {
        identity_index: usize,
        color: ClueColorId,
    },
    UnknownIdentityClueRank {
        identity_index: usize,
        rank: ClueRank,
    },
}

fn ensure_unique<T>(kind: &'static str, values: &[T]) -> Result<(), VariantSpecError>
where
    T: Copy + Eq + IntoValue,
{
    for (index, value) in values.iter().enumerate() {
        if values[..index].contains(value) {
            return Err(VariantSpecError::DuplicateValue {
                kind,
                value: value.into_value(),
            });
        }
    }
    Ok(())
}

trait IntoValue {
    fn into_value(self) -> u8;
}

impl IntoValue for SuitId {
    fn into_value(self) -> u8 {
        self.value()
    }
}

impl IntoValue for CardRank {
    fn into_value(self) -> u8 {
        self.value()
    }
}

impl IntoValue for ClueColorId {
    fn into_value(self) -> u8 {
        self.value()
    }
}

impl IntoValue for ClueRank {
    fn into_value(self) -> u8 {
        self.value()
    }
}

impl fmt::Display for VariantSpecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStackSuits => formatter.write_str("variant requires at least one stack suit"),
            Self::NoCardRanks => formatter.write_str("variant requires at least one card rank"),
            Self::NoIdentities => formatter.write_str("variant requires at least one identity"),
            Self::TooManyIdentities(count) => {
                write!(
                    formatter,
                    "variant has {count} identities; maximum is {MAX_IDENTITIES}"
                )
            }
            Self::ZeroCopies { identity_index } => {
                write!(
                    formatter,
                    "identity {identity_index} has zero physical copies"
                )
            }
            Self::DeckTooLarge => formatter.write_str("variant deck size overflows usize"),
            Self::DuplicateValue { kind, value } => {
                write!(formatter, "duplicate {kind} value {value}")
            }
            Self::DuplicateIdentity { suit, rank } => write!(
                formatter,
                "duplicate physical identity suit {} rank {}",
                suit.value(),
                rank.value()
            ),
            Self::UnknownIdentitySuit {
                identity_index,
                suit,
            } => write!(
                formatter,
                "identity {identity_index} references unknown stack suit {}",
                suit.value()
            ),
            Self::UnknownIdentityCardRank {
                identity_index,
                rank,
            } => write!(
                formatter,
                "identity {identity_index} references unknown card rank {}",
                rank.value()
            ),
            Self::UnknownIdentityColor {
                identity_index,
                color,
            } => write!(
                formatter,
                "identity {identity_index} references unknown clue color {}",
                color.value()
            ),
            Self::UnknownIdentityClueRank {
                identity_index,
                rank,
            } => write!(
                formatter,
                "identity {identity_index} references unknown clue rank {}",
                rank.value()
            ),
        }
    }
}

impl std::error::Error for VariantSpecError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_variant_matches_the_legacy_identity_order_and_deck() {
        let spec = VariantSpec::no_variant();
        assert_eq!(spec.name(), "No Variant");
        assert_eq!(spec.stack_suits().len(), 5);
        assert_eq!(spec.card_ranks().len(), 5);
        assert_eq!(spec.clue_colors().len(), 5);
        assert_eq!(spec.clue_ranks().len(), 5);
        assert_eq!(spec.identities().identities().len(), 25);
        assert_eq!(spec.identities().deck_size(), 50);
        assert_eq!(spec.identities().all_mask(), (1_u64 << 25) - 1);

        for suit in Suit::ALL {
            for rank in Rank::ALL {
                let card = Card::new(suit, rank);
                let identity = spec.standard_identity_id(card).unwrap();
                assert_eq!(identity.index(), card.index());
            }
        }
    }

    #[test]
    fn clue_domains_are_independent_from_physical_suit_and_rank() {
        let spec = VariantSpec::new(
            "Generic",
            vec![SuitId::new(0), SuitId::new(1)],
            vec![CardRank::new(1), CardRank::new(2)],
            vec![ClueColorId::new(0), ClueColorId::new(1)],
            vec![ClueRank::new(1), ClueRank::new(2)],
            vec![
                IdentitySpec {
                    suit: SuitId::new(0),
                    rank: CardRank::new(1),
                    copies: 2,
                    clue_colors: vec![ClueColorId::new(0), ClueColorId::new(1)],
                    clue_ranks: vec![ClueRank::new(1)],
                },
                IdentitySpec {
                    suit: SuitId::new(1),
                    rank: CardRank::new(2),
                    copies: 1,
                    clue_colors: vec![],
                    clue_ranks: vec![ClueRank::new(1), ClueRank::new(2)],
                },
            ],
        )
        .unwrap();

        assert_eq!(
            spec.identities()
                .clue_mask(GenericClue::Color(ClueColorId::new(1))),
            0b01
        );
        assert_eq!(
            spec.identities()
                .clue_mask(GenericClue::Rank(ClueRank::new(1))),
            0b11
        );
        assert_eq!(
            spec.identities()
                .clue_mask(GenericClue::Rank(ClueRank::new(2))),
            0b10
        );
    }

    #[test]
    fn variant_operations_are_deterministic_and_generic() {
        let spec = VariantSpec::no_variant();
        let red_one = spec
            .standard_identity_id(Card::new(Suit::Red, Rank::One))
            .unwrap();
        let red_two = spec
            .standard_identity_id(Card::new(Suit::Red, Rank::Two))
            .unwrap();

        assert_eq!(spec.copies(red_one), Some(3));
        assert!(spec.touches(red_one, GenericClue::Color(ClueColorId::new(0))));
        assert_eq!(spec.successor(red_one), Some(red_two));
        assert!(spec.playable(&[0, 0, 0, 0, 0], red_one));
        assert!(!spec.playable(&[1, 0, 0, 0, 0], red_one));
        assert_eq!(spec.max_score(), 25);
        assert_eq!(
            spec.legal_clues(&[red_one]),
            vec![
                GenericClue::Color(ClueColorId::new(0)),
                GenericClue::Rank(ClueRank::new(1)),
            ]
        );
    }

    #[test]
    fn max_score_stops_at_the_first_missing_stack_rank() {
        let spec = VariantSpec::new(
            "Sparse",
            vec![SuitId::new(0)],
            vec![CardRank::new(1), CardRank::new(2), CardRank::new(3)],
            vec![],
            vec![],
            vec![
                IdentitySpec {
                    suit: SuitId::new(0),
                    rank: CardRank::new(1),
                    copies: 1,
                    clue_colors: vec![],
                    clue_ranks: vec![],
                },
                IdentitySpec {
                    suit: SuitId::new(0),
                    rank: CardRank::new(3),
                    copies: 1,
                    clue_colors: vec![],
                    clue_ranks: vec![],
                },
            ],
        )
        .unwrap();

        assert_eq!(spec.max_score(), 1);
        let one = spec
            .identities()
            .id_for(SuitId::new(0), CardRank::new(1))
            .unwrap();
        assert_eq!(spec.successor(one), None);
    }

    #[test]
    fn universe_rejects_more_than_u64_can_represent() {
        let identities = (0_u8..65)
            .map(|rank| IdentitySpec {
                suit: SuitId::new(0),
                rank: CardRank::new(rank),
                copies: 1,
                clue_colors: vec![],
                clue_ranks: vec![],
            })
            .collect::<Vec<_>>();
        let error = VariantSpec::new(
            "Too Many",
            vec![SuitId::new(0)],
            (0_u8..65).map(CardRank::new).collect(),
            vec![],
            vec![],
            identities,
        )
        .unwrap_err();
        assert_eq!(error, VariantSpecError::TooManyIdentities(65));
    }
}
