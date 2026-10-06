use hanabi_core::{Card, CardId, Clue, ClueFacts, GenericClue, Rank, VariantSpec};

use super::{HGroupClueInterpretation, HGroupClueKind, HGroupSaveKind, IdentitySet};

/// Once every remaining identity has a positively clued, accounted-for copy,
/// clues only spend time. New collateral touches do not create play promises.
/// <https://hanabi.github.io/level-8/#burning-end-game-stalling>
/// Reviewed in p4v0s1 turn 48. Ambiguous cards cannot cover multiple ranks.
pub(super) fn fully_clued_endgame(
    stacks: [u8; 5],
    deck: usize,
    players: usize,
    cards: impl Iterator<Item = (bool, IdentitySet)>,
) -> bool {
    if deck > players {
        return false;
    }
    let covered = cards
        .filter(|(positive, identities)| *positive && identities.len() == 1)
        .fold(IdentitySet::from_mask(0), |all, (_, identities)| {
            all.union(identities)
        });
    hanabi_core::Suit::ALL.into_iter().all(|suit| {
        Rank::ALL.into_iter().all(|rank| {
            rank.number() <= stacks[suit.index()] || covered.contains(Card::new(suit, rank))
        })
    })
}

pub(super) fn strictly_narrows(before: IdentitySet, after: IdentitySet) -> bool {
    !after.is_empty() && after != before && after.intersection(before) == after
}

/// Pre-clue promise narrowed by accumulated literal information. This is
/// deliberately independent of the current physical face and current clue.
pub(super) fn prior_superposition<'a>(
    card: CardId,
    facts: ClueFacts,
    stacks: [u8; 5],
    clues: impl DoubleEndedIterator<Item = &'a HGroupClueInterpretation>,
) -> IdentitySet {
    let literal = IdentitySet::from_mask(facts.identity_mask());
    let Some(prior) = clues.rev().find(|prior| prior.focus == card) else {
        return literal;
    };
    let promised = prior
        .play_identities
        .union(prior.save_identities)
        .intersection(literal);
    if promised.is_empty() {
        return literal;
    }
    let useful = IdentitySet::from_mask(
        promised
            .iter()
            .filter(|identity| identity.rank.number() > stacks[identity.suit.index()])
            .fold(0, |mask, identity| mask | (1 << identity.index())),
    );
    if useful.is_empty() { promised } else { useful }
}

/// Exact identity established for a protected card without looking at its
/// hidden face. Shared by Save and Chop Move value checks.
pub(super) fn protected_identity_from_clues<'a>(
    card: CardId,
    facts: ClueFacts,
    stacks: [u8; 5],
    clues: impl DoubleEndedIterator<Item = &'a HGroupClueInterpretation>,
) -> Option<Card> {
    let literal = IdentitySet::from_mask(facts.identity_mask());
    if literal.len() == 1 {
        return literal.iter().next();
    }
    let remaining = remaining_prior_play_identities(card, facts, stacks, clues);
    (remaining.len() == 1)
        .then(|| remaining.iter().next())
        .flatten()
}

/// Evaluate an existing useful-card promise against the literal information
/// and stacks *before* a reclue. Later fill-ins and completed predecessors can
/// resolve an originally ambiguous Play promise without another Play Clue.
pub(super) fn prior_play_is_ready<'a>(
    card: CardId,
    facts: ClueFacts,
    stacks: [u8; 5],
    clues: impl DoubleEndedIterator<Item = &'a HGroupClueInterpretation>,
) -> bool {
    let remaining = remaining_prior_play_identities(card, facts, stacks, clues);
    !remaining.is_empty()
        && remaining
            .iter()
            .all(|identity| identity.rank.number() == stacks[identity.suit.index()] + 1)
}

/// Remaining identities of an existing useful Play promise, using only the
/// literal facts and stack heights available at the event being interpreted.
pub(super) fn remaining_prior_play_identities<'a>(
    card: CardId,
    facts: ClueFacts,
    stacks: [u8; 5],
    clues: impl DoubleEndedIterator<Item = &'a HGroupClueInterpretation>,
) -> IdentitySet {
    clues
        .rev()
        .find(|clue| clue.focus == card)
        .map_or_else(IdentitySet::default, |clue| {
            IdentitySet::from_mask(
                clue.play_identities
                    .iter()
                    .filter(|identity| {
                        facts.allows(*identity)
                            && identity.rank.number() > stacks[identity.suit.index()]
                    })
                    .fold(0, |mask, identity| mask | (1 << identity.index())),
            )
        })
}

/// A higher-precedence meaning that suppresses an ordinary Play/Save reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PrimarySuppression {
    Fix,
    FiveChopMove,
    LowScoreFive,
    EarlyFiveStall,
    EightClueFiveStall,
    NoInformationReclue,
    AlreadyPlayingReclue,
}

/// Inputs to the one primary clue-precedence resolver.
#[derive(Clone, Copy, Debug)]
pub(super) struct PrimaryClueInputs<'a> {
    pub(super) clue: GenericClue,
    pub(super) variant: &'a VariantSpec,
    pub(super) play_identities: IdentitySet,
    pub(super) save_identities: IdentitySet,
    pub(super) stack_heights: [u8; 5],
    pub(super) eight_clue_save: bool,
    /// Ordered highest-to-lowest precedence overrides that apply to this
    /// clue. An array keeps the hot path allocation-free while avoiding an
    /// error-prone bag of unrelated boolean parameters.
    pub(super) suppressions: [Option<PrimarySuppression>; 7],
}

/// Inputs to the variant-aware Level 1 Save semantic kernel.
///
/// Count slices are indexed by the active variant's `IdentityId`. Visible
/// counts are from the giver's perspective and exclude the focus card itself.
/// `double_chop_twos` marks identities for which every otherwise-visible copy
/// is also on chop, the Level 1 exception to the 2 Save Visible Rule.
#[derive(Clone, Copy, Debug)]
pub(super) struct SaveClueInputs<'a> {
    pub(super) clue: GenericClue,
    pub(super) variant: &'a VariantSpec,
    pub(super) focus_identities: IdentitySet,
    pub(super) focus_was_chop: bool,
    pub(super) stack_heights: &'a [usize],
    pub(super) discarded_counts: &'a [usize],
    pub(super) visible_other_counts: &'a [usize],
    pub(super) double_chop_twos: IdentitySet,
}

/// Canonical primary meaning shared by replay and prospective interpretation.
/// Secondary rules may add connections or explanations, but cannot silently
/// replace this meaning through a separate precedence ladder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ClueInterpretationPlan {
    pub(super) kind: HGroupClueKind,
    pub(super) play_identities: IdentitySet,
    pub(super) save_identities: IdentitySet,
    pub(super) suppression: Option<PrimarySuppression>,
}

impl ClueInterpretationPlan {
    /// Immediate Level 1 Play possibilities only. The domain and stack heights
    /// must belong to the active variant; heights count completed stack ranks.
    /// Save precedence, suppressions, and delayed connections are resolved by
    /// the standard replay path, not by this physical semantic kernel.
    pub(super) fn direct_play(
        variant: &VariantSpec,
        stack_heights: &[usize],
        clue: GenericClue,
        focus_identities: IdentitySet,
    ) -> Self {
        let play_identities = focus_identities
            .filter_clue(variant, clue, true)
            .iter_ids()
            .filter(|identity| variant.playable(stack_heights, *identity))
            .fold(IdentitySet::default(), |set, identity| {
                set.union(IdentitySet::singleton_id(identity))
            });
        Self {
            kind: if play_identities.is_empty() {
                HGroupClueKind::Unrecognized
            } else {
                HGroupClueKind::Play
            },
            play_identities,
            save_identities: IdentitySet::default(),
            suppression: None,
        }
    }

    /// Base Level 1 Save possibilities at clue time.
    ///
    /// This kernel is deliberately independent from historical replay and from
    /// variant-specific higher-level clue-choice overrides. Physical touching,
    /// stack progress, multiplicity, public discards, and the 2 Save Visible
    /// Rule come from the active variant and caller-supplied public snapshot.
    pub(super) fn save(inputs: SaveClueInputs<'_>) -> Self {
        if !inputs.focus_was_chop {
            return Self {
                kind: HGroupClueKind::Unrecognized,
                play_identities: IdentitySet::default(),
                save_identities: IdentitySet::default(),
                suppression: None,
            };
        }

        let save_identities = inputs
            .focus_identities
            .filter_clue(inputs.variant, inputs.clue, true)
            .iter_ids()
            .filter(|identity_id| {
                let Some(identity) = inputs.variant.identities().get(*identity_id) else {
                    return false;
                };
                let Some(suit_index) = inputs
                    .variant
                    .stack_suits()
                    .iter()
                    .position(|suit| *suit == identity.suit)
                else {
                    return false;
                };
                let Some(rank_index) = inputs
                    .variant
                    .card_ranks()
                    .iter()
                    .position(|rank| *rank == identity.rank)
                else {
                    return false;
                };
                let Some(stack_height) = inputs.stack_heights.get(suit_index).copied() else {
                    return false;
                };
                if rank_index < stack_height {
                    return false;
                }

                match inputs.clue {
                    GenericClue::Rank(rank)
                        if rank.value() == 5 && identity.rank.value() == 5 =>
                    {
                        true
                    }
                    GenericClue::Rank(rank)
                        if rank.value() == 2 && identity.rank.value() == 2 =>
                    {
                        inputs
                            .visible_other_counts
                            .get(identity_id.index())
                            .is_some_and(|visible| {
                                *visible == 0
                                    || inputs.double_chop_twos.contains_id(*identity_id)
                            })
                    }
                    GenericClue::Color(_) | GenericClue::Rank(_) => {
                        if identity.rank.value() == 5
                            || inputs.variant.playable(inputs.stack_heights, *identity_id)
                        {
                            return false;
                        }
                        let Some(copies) = inputs.variant.copies(*identity_id).map(usize::from)
                        else {
                            return false;
                        };
                        let Some(discarded) =
                            inputs.discarded_counts.get(identity_id.index()).copied()
                        else {
                            return false;
                        };
                        let Some(visible) =
                            inputs.visible_other_counts.get(identity_id.index()).copied()
                        else {
                            return false;
                        };
                        discarded + 1 == copies && visible == 0
                    }
                }
            })
            .fold(IdentitySet::default(), |set, identity| {
                set.union(IdentitySet::singleton_id(identity))
            });

        let kind = if save_identities.is_empty() {
            HGroupClueKind::Unrecognized
        } else {
            match inputs.clue {
                GenericClue::Rank(rank) if rank.value() == 5 => {
                    HGroupClueKind::Save(HGroupSaveKind::Five)
                }
                GenericClue::Rank(rank) if rank.value() == 2 => {
                    HGroupClueKind::Save(HGroupSaveKind::Two)
                }
                GenericClue::Color(_) | GenericClue::Rank(_) => {
                    HGroupClueKind::Save(HGroupSaveKind::Critical)
                }
            }
        };

        Self {
            kind,
            play_identities: IdentitySet::default(),
            save_identities,
            suppression: None,
        }
    }

    pub(super) fn resolve(inputs: PrimaryClueInputs<'_>) -> Self {
        let clue = super::standard_h_group_clue(inputs.variant, inputs.clue);
        // Rank 2/5 saves and non-playable critical saves take precedence over
        // a hypothetical play reading. This preserves explicit Play-or-Save
        // superpositions only where neither branch outranks the other.
        let save_precedence = IdentitySet::from_mask(
            inputs
                .save_identities
                .iter()
                .filter(|identity| {
                    inputs.eight_clue_save
                        || matches!(
                            (clue, identity.rank),
                            (Clue::Rank(Rank::Two), Rank::Two)
                                | (Clue::Rank(Rank::Five), Rank::Five)
                        )
                        || identity.rank.number() > inputs.stack_heights[identity.suit.index()] + 1
                })
                .fold(0, |mask, identity| mask | (1 << identity.index())),
        );
        let mut play_identities = inputs.play_identities.without(save_precedence);
        let suppression = inputs.suppressions.into_iter().flatten().next();
        let kind = if suppression.is_some() {
            play_identities = IdentitySet::default();
            HGroupClueKind::Unrecognized
        } else if inputs.eight_clue_save && !inputs.save_identities.is_empty() {
            HGroupClueKind::Save(HGroupSaveKind::EightClue)
        } else {
            kind_from_masks(clue, play_identities, inputs.save_identities)
        };
        Self {
            kind,
            play_identities,
            save_identities: inputs.save_identities,
            suppression,
        }
    }
}

fn kind_from_masks(clue: Clue, play: IdentitySet, save: IdentitySet) -> HGroupClueKind {
    match (play.is_empty(), save.is_empty()) {
        (false, false) => HGroupClueKind::PlayOrSave,
        (false, true) => HGroupClueKind::Play,
        (true, false) => match clue {
            Clue::Rank(Rank::Five) => HGroupClueKind::Save(HGroupSaveKind::Five),
            Clue::Rank(Rank::Two) => HGroupClueKind::Save(HGroupSaveKind::Two),
            Clue::Suit(_) | Clue::Rank(_) => HGroupClueKind::Save(HGroupSaveKind::Critical),
        },
        (true, true) => HGroupClueKind::Unrecognized,
    }
}

#[cfg(test)]
mod tests {
    use hanabi_core::{Card, Suit};

    use super::*;

    fn identity(suit: Suit, rank: Rank) -> IdentitySet {
        IdentitySet::singleton(Card::new(suit, rank))
    }

    fn physical_identity(variant: &VariantSpec, suit: u8, rank: u8) -> IdentitySet {
        IdentitySet::singleton_id(
            variant
                .identities()
                .id_for(Suit::new(suit), Rank::new(rank))
                .unwrap(),
        )
    }

    #[test]
    fn direct_play_supports_a_sixth_suit_outside_standard_identities() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Black6Suits);
        let focus = physical_identity(&variant, 5, 2);
        assert!(focus.iter_ids().next().unwrap().value() >= 25);
        let plan = ClueInterpretationPlan::direct_play(
            &variant,
            &[0, 0, 0, 0, 0, 1],
            GenericClue::Color(hanabi_core::ClueColorId::new(5)),
            focus,
        );
        assert_eq!(plan.kind, HGroupClueKind::Play);
        assert_eq!(plan.play_identities, focus);
        assert!(plan.save_identities.is_empty());
        assert_eq!(plan.suppression, None);
    }

    #[test]
    fn direct_play_rainbow_uses_physical_color_touching_and_focus_domain() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Rainbow6Suits);
        let rainbow = physical_identity(&variant, 5, 1);
        let red = physical_identity(&variant, 0, 1);
        let blue = physical_identity(&variant, 3, 1);
        let focus = rainbow.union(red).union(blue);
        for color in variant.clue_colors() {
            let clue = GenericClue::Color(*color);
            assert!(variant.touches(rainbow.iter_ids().next().unwrap(), clue));
            let plan = ClueInterpretationPlan::direct_play(&variant, &[0; 6], clue, focus);
            let expected = match color.value() {
                0 => rainbow.union(red),
                3 => rainbow.union(blue),
                _ => rainbow,
            };
            assert_eq!(plan.kind, HGroupClueKind::Play);
            assert_eq!(plan.play_identities, expected);
        }
    }

    #[test]
    fn direct_play_excludes_touched_nonplayable_and_untouched_playable_identities() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Rainbow6Suits);
        let rainbow_two = physical_identity(&variant, 5, 2);
        let blue_one = physical_identity(&variant, 3, 1);
        let clue = GenericClue::Color(hanabi_core::ClueColorId::new(0));
        assert!(variant.touches(rainbow_two.iter_ids().next().unwrap(), clue));
        for focus in [
            rainbow_two,
            rainbow_two.union(blue_one),
            IdentitySet::default(),
        ] {
            let plan = ClueInterpretationPlan::direct_play(&variant, &[0; 6], clue, focus);
            assert_eq!(plan.kind, HGroupClueKind::Unrecognized);
            assert!(plan.play_identities.is_empty());
        }
        // A completed touched identity cannot promise another immediate play.
        let plan =
            ClueInterpretationPlan::direct_play(&variant, &[0, 0, 0, 0, 0, 2], clue, rainbow_two);
        assert_eq!(plan.kind, HGroupClueKind::Unrecognized);
        assert!(plan.play_identities.is_empty());
    }

    #[test]
    fn direct_play_preserves_no_variant_color_and_rank_parity() {
        let variant = VariantSpec::no_variant();
        for stacks in [[0; 5], [1, 2, 3, 4, 5], [5; 5]] {
            for clue in Suit::ALL
                .into_iter()
                .map(Clue::Suit)
                .chain(Rank::ALL.into_iter().map(Clue::Rank))
            {
                for focus in [IdentitySet::all(), identity(Suit::Red, Rank::Two)] {
                    let expected = IdentitySet::from_mask(
                        focus
                            .iter()
                            .filter(|card| {
                                clue.matches(*card)
                                    && card.rank.number() == stacks[card.suit.index()] + 1
                            })
                            .fold(0, |mask, card| mask | (1 << card.index())),
                    );
                    let plan = ClueInterpretationPlan::direct_play(
                        &variant,
                        &stacks.map(usize::from),
                        GenericClue::from(clue),
                        focus,
                    );
                    assert_eq!(plan.play_identities, expected);
                    assert_eq!(
                        plan.kind,
                        if expected.is_empty() {
                            HGroupClueKind::Unrecognized
                        } else {
                            HGroupClueKind::Play
                        }
                    );
                }
            }
        }
    }

    #[test]
    fn save_kernel_preserves_no_variant_two_five_and_critical_rules() {
        let variant = VariantSpec::no_variant();
        let universe_len = variant.identities().identities().len();
        let discarded = vec![0; universe_len];
        let visible = vec![0; universe_len];

        let red_two = physical_identity(&variant, 0, 2);
        let two = ClueInterpretationPlan::save(SaveClueInputs {
            clue: GenericClue::from(Clue::Rank(Rank::Two)),
            variant: &variant,
            focus_identities: red_two,
            focus_was_chop: true,
            stack_heights: &[1, 0, 0, 0, 0],
            discarded_counts: &discarded,
            visible_other_counts: &visible,
            double_chop_twos: IdentitySet::default(),
        });
        assert_eq!(two.kind, HGroupClueKind::Save(HGroupSaveKind::Two));
        assert_eq!(two.save_identities, red_two);

        let red_five = physical_identity(&variant, 0, 5);
        let five = ClueInterpretationPlan::save(SaveClueInputs {
            clue: GenericClue::from(Clue::Rank(Rank::Five)),
            variant: &variant,
            focus_identities: red_five,
            focus_was_chop: true,
            stack_heights: &[4, 0, 0, 0, 0],
            discarded_counts: &discarded,
            visible_other_counts: &visible,
            double_chop_twos: IdentitySet::default(),
        });
        assert_eq!(five.kind, HGroupClueKind::Save(HGroupSaveKind::Five));
        assert_eq!(five.save_identities, red_five);

        let red_three = variant
            .identities()
            .id_for(hanabi_core::SuitId::new(0), hanabi_core::CardRank::new(3))
            .unwrap();
        let mut critical_discards = discarded.clone();
        critical_discards[red_three.index()] = 1;
        let critical = ClueInterpretationPlan::save(SaveClueInputs {
            clue: GenericClue::Color(hanabi_core::ClueColorId::new(0)),
            variant: &variant,
            focus_identities: IdentitySet::singleton_id(red_three),
            focus_was_chop: true,
            stack_heights: &[1, 0, 0, 0, 0],
            discarded_counts: &critical_discards,
            visible_other_counts: &visible,
            double_chop_twos: IdentitySet::default(),
        });
        assert_eq!(
            critical.kind,
            HGroupClueKind::Save(HGroupSaveKind::Critical)
        );
        assert_eq!(
            critical.save_identities,
            IdentitySet::singleton_id(red_three)
        );

        let playable_critical = ClueInterpretationPlan::save(SaveClueInputs {
            stack_heights: &[2, 0, 0, 0, 0],
            ..SaveClueInputs {
                clue: GenericClue::Color(hanabi_core::ClueColorId::new(0)),
                variant: &variant,
                focus_identities: IdentitySet::singleton_id(red_three),
                focus_was_chop: true,
                stack_heights: &[0; 5],
                discarded_counts: &critical_discards,
                visible_other_counts: &visible,
                double_chop_twos: IdentitySet::default(),
            }
        });
        assert_eq!(playable_critical.kind, HGroupClueKind::Unrecognized);
        assert!(playable_critical.save_identities.is_empty());
    }

    #[test]
    fn save_kernel_respects_two_save_visible_rule_and_double_chop_exception() {
        let variant = VariantSpec::no_variant();
        let red_two = variant
            .identities()
            .id_for(hanabi_core::SuitId::new(0), hanabi_core::CardRank::new(2))
            .unwrap();
        let focus = IdentitySet::singleton_id(red_two);
        let discarded = vec![0; variant.identities().identities().len()];
        let mut visible = discarded.clone();
        visible[red_two.index()] = 1;

        let blocked = ClueInterpretationPlan::save(SaveClueInputs {
            clue: GenericClue::from(Clue::Rank(Rank::Two)),
            variant: &variant,
            focus_identities: focus,
            focus_was_chop: true,
            stack_heights: &[0; 5],
            discarded_counts: &discarded,
            visible_other_counts: &visible,
            double_chop_twos: IdentitySet::default(),
        });
        assert_eq!(blocked.kind, HGroupClueKind::Unrecognized);

        let double_chop = ClueInterpretationPlan::save(SaveClueInputs {
            double_chop_twos: focus,
            ..SaveClueInputs {
                clue: GenericClue::from(Clue::Rank(Rank::Two)),
                variant: &variant,
                focus_identities: focus,
                focus_was_chop: true,
                stack_heights: &[0; 5],
                discarded_counts: &discarded,
                visible_other_counts: &visible,
                double_chop_twos: IdentitySet::default(),
            }
        });
        assert_eq!(
            double_chop.kind,
            HGroupClueKind::Save(HGroupSaveKind::Two)
        );

        let off_chop = ClueInterpretationPlan::save(SaveClueInputs {
            focus_was_chop: false,
            ..SaveClueInputs {
                clue: GenericClue::from(Clue::Rank(Rank::Two)),
                variant: &variant,
                focus_identities: focus,
                focus_was_chop: true,
                stack_heights: &[0; 5],
                discarded_counts: &discarded,
                visible_other_counts: &visible,
                double_chop_twos: focus,
            }
        });
        assert_eq!(off_chop.kind, HGroupClueKind::Unrecognized);
    }

    #[test]
    fn save_kernel_uses_black_one_copy_multiplicity_without_legacy_rank_copies() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Black6Suits);
        let black_three = variant
            .identities()
            .id_for(hanabi_core::SuitId::new(5), hanabi_core::CardRank::new(3))
            .unwrap();
        assert!(black_three.value() >= 25);
        assert_eq!(variant.copies(black_three), Some(1));

        let counts = vec![0; variant.identities().identities().len()];
        let plan = ClueInterpretationPlan::save(SaveClueInputs {
            clue: GenericClue::Color(hanabi_core::ClueColorId::new(5)),
            variant: &variant,
            focus_identities: IdentitySet::singleton_id(black_three),
            focus_was_chop: true,
            stack_heights: &[0; 6],
            discarded_counts: &counts,
            visible_other_counts: &counts,
            double_chop_twos: IdentitySet::default(),
        });
        assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Critical));
        assert_eq!(
            plan.save_identities,
            IdentitySet::singleton_id(black_three)
        );
    }

    #[test]
    fn save_kernel_uses_variant_touching_for_rainbow_critical_saves() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Rainbow6Suits);
        let rainbow_three = variant
            .identities()
            .id_for(hanabi_core::SuitId::new(5), hanabi_core::CardRank::new(3))
            .unwrap();
        let mut discarded = vec![0; variant.identities().identities().len()];
        discarded[rainbow_three.index()] = 1;
        let visible = vec![0; discarded.len()];
        let focus = IdentitySet::singleton_id(rainbow_three);

        for color in variant.clue_colors() {
            let clue = GenericClue::Color(*color);
            assert!(variant.touches(rainbow_three, clue));
            let plan = ClueInterpretationPlan::save(SaveClueInputs {
                clue,
                variant: &variant,
                focus_identities: focus,
                focus_was_chop: true,
                stack_heights: &[0; 6],
                discarded_counts: &discarded,
                visible_other_counts: &visible,
                double_chop_twos: IdentitySet::default(),
            });
            assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Critical));
            assert_eq!(plan.save_identities, focus);
        }
    }

    #[test]
    fn played_domains_cannot_change_endgame_coverage_when_refined() {
        let stacks = [4; 5];
        let dead = identity(Suit::Red, Rank::One).union(identity(Suit::Blue, Rank::Four));
        for covered_suits in 0..=5 {
            let covered = Suit::ALL[..covered_suits]
                .iter()
                .map(|suit| (true, identity(*suit, Rank::Five)))
                .collect::<Vec<_>>();
            let check = |domain| {
                fully_clued_endgame(
                    stacks,
                    0,
                    4,
                    covered.iter().copied().chain([(true, domain)]),
                )
            };
            assert_eq!(check(dead), covered_suits == 5);
            for card in dead.iter() {
                assert_eq!(check(IdentitySet::singleton(card)), check(dead));
            }
        }
    }

    #[test]
    fn information_gain_requires_a_nonempty_strict_subset() {
        let four = identity(Suit::Purple, Rank::Four);
        let five = identity(Suit::Purple, Rank::Five);
        let both = four.union(five);
        assert!(strictly_narrows(both, four));
        assert!(!strictly_narrows(four, four));
        assert!(!strictly_narrows(four, five));
        assert!(!strictly_narrows(four, both));
        assert!(!strictly_narrows(four, IdentitySet::default()));
    }

    #[test]
    fn five_chop_move_suppresses_a_hypothetical_play_reading() {
        let plan = ClueInterpretationPlan::resolve(PrimaryClueInputs {
            clue: GenericClue::from(Clue::Rank(Rank::Five)),
            variant: &VariantSpec::no_variant(),
            play_identities: identity(Suit::Red, Rank::Five),
            save_identities: IdentitySet::default(),
            stack_heights: [4; 5],
            eight_clue_save: false,
            suppressions: [
                None,
                Some(PrimarySuppression::FiveChopMove),
                None,
                None,
                None,
                None,
                None,
            ],
        });
        assert_eq!(plan.kind, HGroupClueKind::Unrecognized);
        assert!(plan.play_identities.is_empty());
        assert_eq!(plan.suppression, Some(PrimarySuppression::FiveChopMove));
    }

    #[test]
    fn playable_rank_two_on_chop_remains_a_save() {
        let red_two = identity(Suit::Red, Rank::Two);
        let plan = ClueInterpretationPlan::resolve(PrimaryClueInputs {
            clue: GenericClue::from(Clue::Rank(Rank::Two)),
            variant: &VariantSpec::no_variant(),
            play_identities: red_two,
            save_identities: red_two,
            stack_heights: [1, 0, 0, 0, 0],
            eight_clue_save: false,
            suppressions: [None; 7],
        });
        assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Two));
        assert!(plan.play_identities.is_empty());
    }
}
