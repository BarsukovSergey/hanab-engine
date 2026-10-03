//! Physical catalog facts independently modeled from Hanabi Live. See SOURCES.md.

use super::{CardRank, ClueColorId, ClueRank, IdentitySpec, SuitId, VariantSpec};

/// Revision of Hanabi Live used for the checked-in physical catalog.
pub const HANABI_LIVE_VARIANT_REVISION: &str = "67517fcce0a441bf75313d92dd6078d34495971b";

/// Engine-owned catalog keys, independent of Hanabi Live protocol identifiers.
///
/// Special variants explicitly denote the six-suit edition: the five standard
/// suits followed by the special suit. Suit and clue color IDs are spec-local.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VariantId {
    NoVariant,
    Black6Suits,
    Rainbow6Suits,
    White6Suits,
    Pink6Suits,
    Brown6Suits,
    Prism6Suits,
    Omni6Suits,
    Null6Suits,
}

impl VariantId {
    pub const ALL: [Self; 9] = [
        Self::NoVariant,
        Self::Black6Suits,
        Self::Rainbow6Suits,
        Self::White6Suits,
        Self::Pink6Suits,
        Self::Brown6Suits,
        Self::Prism6Suits,
        Self::Omni6Suits,
        Self::Null6Suits,
    ];

    /// Stable key suitable for engine configuration and persistence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.definition().key
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        self.definition().name
    }

    const fn definition(self) -> &'static CatalogEntry {
        &CATALOG[self as usize]
    }
}

#[derive(Clone, Copy)]
enum ColorTouch {
    Own,
    All,
    None,
    ByRank,
}

#[derive(Clone, Copy)]
enum RankTouch {
    Own,
    All,
    None,
}

struct SuitDefinition {
    copies: [u8; 5],
    colors: ColorTouch,
    ranks: RankTouch,
}

struct CatalogEntry {
    key: &'static str,
    name: &'static str,
    extra_suit: Option<SuitDefinition>,
}

const STANDARD_COPIES: [u8; 5] = [3, 2, 2, 2, 1];
const STANDARD_SUIT: SuitDefinition = SuitDefinition {
    copies: STANDARD_COPIES,
    colors: ColorTouch::Own,
    ranks: RankTouch::Own,
};

const CATALOG: [CatalogEntry; 9] = [
    CatalogEntry {
        key: "no-variant",
        name: "No Variant",
        extra_suit: None,
    },
    CatalogEntry {
        key: "black-6-suits",
        name: "Black (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: [1; 5],
            colors: ColorTouch::Own,
            ranks: RankTouch::Own,
        }),
    },
    CatalogEntry {
        key: "rainbow-6-suits",
        name: "Rainbow (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::All,
            ranks: RankTouch::Own,
        }),
    },
    CatalogEntry {
        key: "white-6-suits",
        name: "White (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::None,
            ranks: RankTouch::Own,
        }),
    },
    CatalogEntry {
        key: "pink-6-suits",
        name: "Pink (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::Own,
            ranks: RankTouch::All,
        }),
    },
    CatalogEntry {
        key: "brown-6-suits",
        name: "Brown (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::Own,
            ranks: RankTouch::None,
        }),
    },
    CatalogEntry {
        key: "prism-6-suits",
        name: "Prism (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::ByRank,
            ranks: RankTouch::Own,
        }),
    },
    CatalogEntry {
        key: "omni-6-suits",
        name: "Omni (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::All,
            ranks: RankTouch::All,
        }),
    },
    CatalogEntry {
        key: "null-6-suits",
        name: "Null (6 Suits)",
        extra_suit: Some(SuitDefinition {
            copies: STANDARD_COPIES,
            colors: ColorTouch::None,
            ranks: RankTouch::None,
        }),
    },
];

impl VariantSpec {
    /// Builds a catalog variant without fetching data or interpreting its name.
    ///
    /// # Panics
    ///
    /// Panics if checked-in catalog data violates `Self::new` validation.
    #[must_use]
    pub fn from_catalog(id: VariantId) -> Self {
        let definition = id.definition();
        let mut suits = vec![&STANDARD_SUIT; 5];
        suits.extend(definition.extra_suit.as_ref());
        let stack_suits = (0..suits.len())
            .map(|index| SuitId::new(u8::try_from(index).expect("at most six suits")))
            .collect::<Vec<_>>();
        let clue_colors = suits
            .iter()
            .zip(&stack_suits)
            .filter(|(suit, _)| matches!(suit.colors, ColorTouch::Own))
            .map(|(_, suit)| ClueColorId::new(suit.value()))
            .collect::<Vec<_>>();
        let clue_ranks = (1..=5).map(ClueRank::new).collect::<Vec<_>>();
        let mut identities = Vec::with_capacity(suits.len() * 5);
        for (suit, physical) in stack_suits.iter().copied().zip(suits) {
            for rank in CardRank::ALL {
                let colors = match physical.colors {
                    ColorTouch::Own => vec![ClueColorId::new(suit.value())],
                    ColorTouch::All => clue_colors.clone(),
                    ColorTouch::None => vec![],
                    ColorTouch::ByRank => vec![clue_colors[rank.index() % clue_colors.len()]],
                };
                let ranks = match physical.ranks {
                    RankTouch::Own => vec![ClueRank::new(rank.value())],
                    RankTouch::All => clue_ranks.clone(),
                    RankTouch::None => vec![],
                };
                identities.push(IdentitySpec {
                    suit,
                    rank,
                    copies: physical.copies[rank.index()],
                    clue_colors: colors,
                    clue_ranks: ranks,
                });
            }
        }
        let mut spec = Self::new(
            definition.name,
            stack_suits,
            CardRank::ALL.to_vec(),
            clue_colors,
            clue_ranks,
            identities,
        )
        .expect("built-in physical catalog is valid");
        spec.catalog_id = Some(id);
        spec
    }
}
