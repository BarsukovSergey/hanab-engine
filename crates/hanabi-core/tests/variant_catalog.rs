//! Physical-rule tests, not convention or strategy authorities.
use hanabi_core::{
    CardRank, ClueColorId, ClueRank, GenericClue, HANABI_LIVE_VARIANT_REVISION, IdentityId,
    IdentitySpec, SuitId, VariantId, VariantSpec,
};

struct Expected {
    id: VariantId,
    key: &'static str,
    name: &'static str,
    suits: u8,
    deck: usize,
    colors: u8,
    extra_copies: [u8; 5],
    extra_colors: [u8; 5],
    extra_ranks: [u8; 5],
    score: usize,
}

fn check_physical_rules(expected: &Expected) {
    let spec = VariantSpec::from_catalog(expected.id);
    assert_eq!(spec.catalog_id(), Some(expected.id));
    assert_eq!(expected.id.as_str(), expected.key);
    assert_eq!(expected.id.name(), expected.name);
    assert_eq!(spec.name(), expected.name);
    assert_eq!(
        spec.stack_suits(),
        &(0..expected.suits).map(SuitId::new).collect::<Vec<_>>()
    );
    assert_eq!(spec.card_ranks(), &CardRank::ALL);
    assert_eq!(
        spec.clue_colors(),
        &(0..expected.colors)
            .map(ClueColorId::new)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        spec.clue_ranks(),
        &(1..=5).map(ClueRank::new).collect::<Vec<_>>()
    );
    assert_eq!(
        spec.identities().identities().len(),
        usize::from(expected.suits) * 5
    );
    assert_eq!(spec.identities().deck_size(), expected.deck);
    assert_eq!(spec.max_score(), expected.score);

    let all_ids = spec
        .identities()
        .identities()
        .iter()
        .map(|identity| identity.id)
        .collect::<Vec<_>>();
    let domain = spec
        .clue_colors()
        .iter()
        .copied()
        .map(GenericClue::Color)
        .chain(spec.clue_ranks().iter().copied().map(GenericClue::Rank))
        .collect::<Vec<_>>();
    assert_eq!(spec.legal_clues(&all_ids), domain);
    assert_eq!(spec.legal_clues(&[]), Vec::<GenericClue>::new());
    let mut deck = Vec::new();
    for suit in 0..expected.suits {
        for rank in 1..=5 {
            let rank_index = usize::from(rank - 1);
            let identity = spec
                .identities()
                .id_for(SuitId::new(suit), CardRank::new(rank))
                .unwrap();
            assert_eq!(identity.value(), suit * 5 + rank - 1);
            let copies = if suit < 5 {
                [3, 2, 2, 2, 1][rank_index]
            } else {
                expected.extra_copies[rank_index]
            };
            assert_eq!(spec.copies(identity), Some(copies));
            deck.extend(core::iter::repeat_n(identity, usize::from(copies)));
            check_touching(&spec, expected, &domain, identity);

            for height in 0..=5 {
                let mut heights = vec![0; usize::from(expected.suits)];
                heights[usize::from(suit)] = height;
                assert_eq!(spec.playable(&heights, identity), height == rank_index);
            }
            // Progress in other stacks cannot enable this stack's rank 2.
            let mut other_heights = vec![1; usize::from(expected.suits)];
            other_heights[usize::from(suit)] = 0;
            assert_eq!(spec.playable(&other_heights, identity), rank == 1);
            assert!(!spec.playable(&[], identity));
            let successor = if rank == 5 {
                None
            } else {
                spec.identities()
                    .id_for(SuitId::new(suit), CardRank::new(rank + 1))
            };
            assert_eq!(spec.successor(identity), successor);
        }
    }
    assert_eq!(deck.len(), expected.deck);
    for identity in &all_ids {
        assert_eq!(
            deck.iter().filter(|card| *card == identity).count(),
            usize::from(spec.copies(*identity).unwrap())
        );
    }
    let invalid = IdentityId::new(63);
    assert_eq!(spec.copies(invalid), None);
    assert_eq!(spec.successor(invalid), None);
    assert!(!spec.playable(&[0; 6], invalid));
    assert_eq!(spec.legal_clues(&[invalid]), Vec::<GenericClue>::new());
}

fn check_touching(
    spec: &VariantSpec,
    expected: &Expected,
    domain: &[GenericClue],
    identity: IdentityId,
) {
    let card = spec.identities().get(identity).unwrap();
    let suit = card.suit.value();
    let rank = card.rank.value();
    let rank_index = card.rank.index();
    let color_mask = if suit < 5 {
        1 << suit
    } else {
        expected.extra_colors[rank_index]
    };
    let rank_mask = if suit < 5 {
        1 << (rank - 1)
    } else {
        expected.extra_ranks[rank_index]
    };
    let expected_clues = domain
        .iter()
        .copied()
        .filter(|clue| match clue {
            GenericClue::Color(color) => color_mask & (1 << color.value()) != 0,
            GenericClue::Rank(rank) => rank_mask & (1 << (rank.value() - 1)) != 0,
        })
        .collect::<Vec<_>>();
    assert_eq!(spec.legal_clues(&[identity]), expected_clues);
    for clue in domain {
        let touched = expected_clues.contains(clue);
        assert_eq!(
            spec.touches(identity, *clue),
            touched,
            "{expected_name}: suit {suit} rank {rank} clue {clue:?}",
            expected_name = expected.name
        );
        assert_eq!(
            spec.identities().clue_mask(*clue) & (1 << identity.value()) != 0,
            touched
        );
    }
    assert!(!spec.touches(
        identity,
        GenericClue::Color(ClueColorId::new(expected.colors))
    ));
    assert!(!spec.touches(identity, GenericClue::Rank(ClueRank::new(0))));
    assert!(!spec.touches(identity, GenericClue::Rank(ClueRank::new(6))));
}

macro_rules! physical_test {
    ($test:ident, $id:ident, $key:literal, $name:literal, $suits:literal, $deck:literal, $colors:literal, $copies:expr, $touch_colors:expr, $touch_ranks:expr, $score:literal) => {
        #[test]
        fn $test() {
            check_physical_rules(&Expected {
                id: VariantId::$id,
                key: $key,
                name: $name,
                suits: $suits,
                deck: $deck,
                colors: $colors,
                extra_copies: $copies,
                extra_colors: $touch_colors,
                extra_ranks: $touch_ranks,
                score: $score,
            });
        }
    };
}

physical_test!(
    no_variant,
    NoVariant,
    "no-variant",
    "No Variant",
    5,
    50,
    5,
    [0; 5],
    [0; 5],
    [0; 5],
    25
);
physical_test!(
    black,
    Black6Suits,
    "black-6-suits",
    "Black (6 Suits)",
    6,
    55,
    6,
    [1; 5],
    [32; 5],
    [1, 2, 4, 8, 16],
    30
);
physical_test!(
    rainbow,
    Rainbow6Suits,
    "rainbow-6-suits",
    "Rainbow (6 Suits)",
    6,
    60,
    5,
    [3, 2, 2, 2, 1],
    [31; 5],
    [1, 2, 4, 8, 16],
    30
);
physical_test!(
    white,
    White6Suits,
    "white-6-suits",
    "White (6 Suits)",
    6,
    60,
    5,
    [3, 2, 2, 2, 1],
    [0; 5],
    [1, 2, 4, 8, 16],
    30
);
physical_test!(
    pink,
    Pink6Suits,
    "pink-6-suits",
    "Pink (6 Suits)",
    6,
    60,
    6,
    [3, 2, 2, 2, 1],
    [32; 5],
    [31; 5],
    30
);
physical_test!(
    brown,
    Brown6Suits,
    "brown-6-suits",
    "Brown (6 Suits)",
    6,
    60,
    6,
    [3, 2, 2, 2, 1],
    [32; 5],
    [0; 5],
    30
);
physical_test!(
    prism,
    Prism6Suits,
    "prism-6-suits",
    "Prism (6 Suits)",
    6,
    60,
    5,
    [3, 2, 2, 2, 1],
    [1, 2, 4, 8, 16],
    [1, 2, 4, 8, 16],
    30
);
physical_test!(
    omni,
    Omni6Suits,
    "omni-6-suits",
    "Omni (6 Suits)",
    6,
    60,
    5,
    [3, 2, 2, 2, 1],
    [31; 5],
    [31; 5],
    30
);
physical_test!(
    null,
    Null6Suits,
    "null-6-suits",
    "Null (6 Suits)",
    6,
    60,
    5,
    [3, 2, 2, 2, 1],
    [0; 5],
    [0; 5],
    30
);

#[test]
fn catalog_keys_are_unique_and_no_variant_is_compatible() {
    assert_eq!(
        HANABI_LIVE_VARIANT_REVISION,
        "67517fcce0a441bf75313d92dd6078d34495971b"
    );
    for (index, id) in VariantId::ALL.iter().enumerate() {
        assert!(
            !VariantId::ALL[..index]
                .iter()
                .any(|other| other.as_str() == id.as_str())
        );
    }
    assert_eq!(
        VariantSpec::no_variant(),
        VariantSpec::from_catalog(VariantId::NoVariant)
    );
}

#[test]
fn custom_names_do_not_select_catalog_rules() {
    let spec = VariantSpec::new(
        "Black (6 Suits)",
        vec![SuitId::new(0)],
        vec![CardRank::new(1)],
        vec![],
        vec![],
        vec![IdentitySpec {
            suit: SuitId::new(0),
            rank: CardRank::new(1),
            copies: 4,
            clue_colors: vec![],
            clue_ranks: vec![],
        }],
    )
    .unwrap();
    assert_eq!(spec.catalog_id(), None);
    assert_eq!(spec.identities().deck_size(), 4);
    assert_eq!(spec.max_score(), 1);
}
