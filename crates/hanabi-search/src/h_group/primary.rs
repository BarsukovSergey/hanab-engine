use hanabi_core::{Card, CardId, ClueFacts, GenericClue, Rank, VariantSpec};

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
    pub(super) stack_heights: &'a [usize],
    pub(super) eight_clue_save: bool,
    /// Ordered highest-to-lowest precedence overrides that apply to this
    /// clue. An array keeps the hot path allocation-free while avoiding an
    /// error-prone bag of unrelated boolean parameters.
    pub(super) suppressions: [Option<PrimarySuppression>; 7],
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

/// Public snapshot inputs only; counts are indexed by `IdentityId::index()`.
/// Domains, heights, and count slices must belong to the active variant.
#[derive(Clone, Copy, Debug)]
pub(super) struct SaveClueInputs<'a> {
    pub(super) clue: GenericClue,
    pub(super) variant: &'a VariantSpec,
    pub(super) focus_identities: IdentitySet,
    pub(super) focus_was_chop: bool,
    pub(super) stack_heights: &'a [usize],
    pub(super) discarded_counts: &'a [usize],
    /// Other copies visible for the 2 Save Visible Rule, excluding the focus.
    pub(super) two_save_visible_other_counts: &'a [usize],
    /// Subset of visible other copies on chop, for Level 1 Double Chop 2.
    pub(super) two_save_visible_other_chop_counts: &'a [usize],
    /// Other visible physical copies for Critical Save, excluding the focus.
    pub(super) critical_visible_other_counts: &'a [usize],
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

    /// Physical/base Level 1 Save possibilities, without contextual clue-choice
    /// or Loaded conventions. Shared by replay and prospective Save scoring.
    pub(super) fn direct_save(inputs: SaveClueInputs<'_>) -> Self {
        let mut plan = Self {
            kind: HGroupClueKind::Unrecognized,
            play_identities: IdentitySet::default(),
            save_identities: IdentitySet::default(),
            suppression: None,
        };
        if !inputs.focus_was_chop {
            return plan;
        }
        for id in inputs
            .focus_identities
            .filter_clue(inputs.variant, inputs.clue, true)
            .iter_ids()
        {
            let identity = inputs.variant.identities().get(id).unwrap();
            let suit_index = inputs
                .variant
                .stack_suits()
                .iter()
                .position(|suit| *suit == identity.suit)
                .unwrap();
            let rank_index = inputs
                .variant
                .card_ranks()
                .iter()
                .position(|rank| *rank == identity.rank)
                .unwrap();
            if rank_index < inputs.stack_heights[suit_index] {
                continue;
            }
            let visible = inputs.two_save_visible_other_counts[id.index()];
            let kind = if identity.rank.number() == 5
                && matches!(inputs.clue, GenericClue::Rank(rank) if rank.value() == 5)
            {
                HGroupSaveKind::Five
            } else if identity.rank.number() == 2
                && matches!(inputs.clue, GenericClue::Rank(rank) if rank.value() == 2)
                && (visible == 0
                    || visible == inputs.two_save_visible_other_chop_counts[id.index()])
            {
                HGroupSaveKind::Two
            } else if identity.rank.number() != 5
                && !inputs.variant.playable(inputs.stack_heights, id)
                && inputs.discarded_counts[id.index()] + 1
                    == usize::from(inputs.variant.copies(id).unwrap())
                && inputs.critical_visible_other_counts[id.index()] == 0
            {
                HGroupSaveKind::Critical
            } else {
                continue;
            };
            plan.save_identities = plan.save_identities.union(IdentitySet::singleton_id(id));
            // Explicit rank saves outrank a critical possibility touched by
            // the same physical clue in a variant with broader touching.
            if plan.kind == HGroupClueKind::Unrecognized || kind != HGroupSaveKind::Critical {
                plan.kind = HGroupClueKind::Save(kind);
            }
        }
        plan
    }

    pub(super) fn resolve(inputs: PrimaryClueInputs<'_>) -> Self {
        let clue = inputs.clue;
        // Rank 2/5 saves and non-playable critical saves take precedence over
        // a hypothetical play reading. This preserves explicit Play-or-Save
        // superpositions only where neither branch outranks the other.
        let save_precedence = inputs
            .save_identities
            .iter_ids()
            .filter(|id| {
                let identity = inputs
                    .variant
                    .identities()
                    .get(*id)
                    .expect("active identity");
                let suit = inputs
                    .variant
                    .stack_suits()
                    .iter()
                    .position(|suit| *suit == identity.suit)
                    .expect("active suit");
                let rank = inputs
                    .variant
                    .card_ranks()
                    .iter()
                    .position(|rank| *rank == identity.rank)
                    .expect("active rank");
                let explicit_rank_save = matches!(
                    clue,
                    GenericClue::Rank(clue_rank)
                        if matches!(identity.rank.number(), 2 | 5)
                            && clue_rank.value() == identity.rank.number()
                );
                inputs.eight_clue_save || explicit_rank_save || rank > inputs.stack_heights[suit]
            })
            .fold(IdentitySet::default(), |set, id| {
                set.union(IdentitySet::singleton_id(id))
            });
        let mut play_identities = inputs.play_identities.without(save_precedence);
        let suppression = inputs.suppressions.into_iter().flatten().next();
        let kind = if suppression.is_some() {
            play_identities = IdentitySet::default();
            HGroupClueKind::Unrecognized
        } else if inputs.eight_clue_save && !inputs.save_identities.is_empty() {
            HGroupClueKind::Save(HGroupSaveKind::EightClue)
        } else {
            kind_from_masks(
                inputs.variant,
                clue,
                play_identities,
                inputs.save_identities,
            )
        };
        Self {
            kind,
            play_identities,
            save_identities: inputs.save_identities,
            suppression,
        }
    }
}

fn kind_from_masks(
    variant: &VariantSpec,
    clue: GenericClue,
    play: IdentitySet,
    save: IdentitySet,
) -> HGroupClueKind {
    let saves_rank = |rank| {
        save.iter_ids().any(|id| {
            variant
                .identities()
                .get(id)
                .is_some_and(|identity| identity.rank.number() == rank)
        })
    };
    match (play.is_empty(), save.is_empty()) {
        (false, false) => HGroupClueKind::PlayOrSave,
        (false, true) => HGroupClueKind::Play,
        (true, false) => match clue {
            GenericClue::Rank(rank) if rank.value() == 5 && saves_rank(5) => {
                HGroupClueKind::Save(HGroupSaveKind::Five)
            }
            GenericClue::Rank(rank) if rank.value() == 2 && saves_rank(2) => {
                HGroupClueKind::Save(HGroupSaveKind::Two)
            }
            GenericClue::Color(_) | GenericClue::Rank(_) => {
                HGroupClueKind::Save(HGroupSaveKind::Critical)
            }
        },
        (true, true) => HGroupClueKind::Unrecognized,
    }
}

#[cfg(test)]
mod tests {
    use hanabi_core::{Card, Clue, Suit};

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
    #[allow(clippy::too_many_lines)]
    fn primary_resolution_composes_base_play_and_save_with_save_precedence() {
        let standard = VariantSpec::no_variant();
        let rainbow = VariantSpec::from_catalog(hanabi_core::VariantId::Rainbow6Suits);
        let rank = |value| GenericClue::Rank(hanabi_core::ClueRank::new(value));
        let red_two = physical_identity(&standard, 0, 2);
        let red_five = physical_identity(&standard, 0, 5);
        let red_three = physical_identity(&standard, 0, 3);
        let blue_three = physical_identity(&standard, 3, 3);
        let red_one = physical_identity(&rainbow, 0, 1);
        let rainbow_three = physical_identity(&rainbow, 5, 3);
        let empty = IdentitySet::default();
        // name, variant, clue, focus, chop, heights, critical identities,
        // expected kind, expected Play identities, expected Save identities.
        let cases = [
            (
                "playable rank-2 Save",
                &standard,
                rank(2),
                red_two,
                true,
                vec![1, 0, 0, 0, 0],
                empty,
                HGroupClueKind::Save(HGroupSaveKind::Two),
                empty,
                red_two,
            ),
            (
                "playable rank-5 Save",
                &standard,
                rank(5),
                red_five,
                true,
                vec![4, 0, 0, 0, 0],
                empty,
                HGroupClueKind::Save(HGroupSaveKind::Five),
                empty,
                red_five,
            ),
            (
                "ambiguous rank-3 Play or Critical Save",
                &standard,
                rank(3),
                red_three.union(blue_three),
                true,
                vec![2, 0, 0, 0, 0],
                red_three.union(blue_three),
                HGroupClueKind::PlayOrSave,
                red_three,
                blue_three,
            ),
            (
                "off-chop immediate Play",
                &standard,
                rank(2),
                red_two,
                false,
                vec![1, 0, 0, 0, 0],
                empty,
                HGroupClueKind::Play,
                red_two,
                empty,
            ),
            (
                "Rainbow mixed physical color touch",
                &rainbow,
                GenericClue::Color(hanabi_core::ClueColorId::new(0)),
                red_one.union(rainbow_three),
                true,
                vec![0; 6],
                rainbow_three,
                HGroupClueKind::PlayOrSave,
                red_one,
                rainbow_three,
            ),
            (
                "touched but neither playable nor savable",
                &standard,
                rank(3),
                red_three,
                true,
                vec![0; 5],
                empty,
                HGroupClueKind::Unrecognized,
                empty,
                empty,
            ),
        ];
        for (name, variant, clue, focus, chop, heights, critical, kind, play, save) in cases {
            let mut discards = vec![0; variant.stack_suits().len() * variant.card_ranks().len()];
            let visible = vec![0; variant.stack_suits().len() * variant.card_ranks().len()];
            for id in critical.iter_ids() {
                discards[id.index()] = usize::from(variant.copies(id).unwrap()) - 1;
            }
            let save_plan = ClueInterpretationPlan::direct_save(SaveClueInputs {
                clue,
                variant,
                focus_identities: focus,
                focus_was_chop: chop,
                stack_heights: &heights,
                discarded_counts: &discards,
                two_save_visible_other_counts: &visible,
                two_save_visible_other_chop_counts: &visible,
                critical_visible_other_counts: &visible,
            });
            let play_plan = ClueInterpretationPlan::direct_play(variant, &heights, clue, focus);
            let plan = ClueInterpretationPlan::resolve(PrimaryClueInputs {
                clue,
                variant,
                play_identities: play_plan.play_identities,
                save_identities: save_plan.save_identities,
                stack_heights: &heights,
                eight_clue_save: false,
                suppressions: [None; 7],
            });
            assert_eq!(plan.kind, kind, "{name}");
            assert_eq!(plan.play_identities, play, "{name}");
            assert_eq!(plan.save_identities, save, "{name}");
            assert_eq!(plan.suppression, None, "{name}");
        }
    }

    #[test]
    fn primary_resolution_preserves_generic_delayed_play_and_contextual_precedence() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Black6Suits);
        let three = physical_identity(&variant, 5, 3);
        let four = physical_identity(&variant, 5, 4);
        // Replay may supply a delayed connection identity, not just direct_play.
        let inputs = PrimaryClueInputs {
            clue: GenericClue::Color(hanabi_core::ClueColorId::new(5)),
            variant: &variant,
            play_identities: three.union(four),
            save_identities: three,
            stack_heights: &[0; 6],
            eight_clue_save: false,
            suppressions: [None; 7],
        };
        let plan = ClueInterpretationPlan::resolve(inputs);
        assert_eq!(plan.kind, HGroupClueKind::PlayOrSave);
        assert_eq!(plan.play_identities, four);
        assert_eq!(plan.save_identities, three);
        let critical = ClueInterpretationPlan::resolve(PrimaryClueInputs {
            play_identities: three,
            ..inputs
        });
        assert_eq!(
            critical.kind,
            HGroupClueKind::Save(HGroupSaveKind::Critical)
        );
        assert!(critical.play_identities.is_empty());
        let eight = ClueInterpretationPlan::resolve(PrimaryClueInputs {
            eight_clue_save: true,
            ..inputs
        });
        assert_eq!(eight.kind, HGroupClueKind::Save(HGroupSaveKind::EightClue));
        assert_eq!(eight.play_identities, four);
        for suppression in [
            PrimarySuppression::Fix,
            PrimarySuppression::FiveChopMove,
            PrimarySuppression::LowScoreFive,
            PrimarySuppression::EarlyFiveStall,
            PrimarySuppression::EightClueFiveStall,
            PrimarySuppression::NoInformationReclue,
            PrimarySuppression::AlreadyPlayingReclue,
        ] {
            let suppressed = ClueInterpretationPlan::resolve(PrimaryClueInputs {
                suppressions: [Some(suppression); 7],
                ..inputs
            });
            assert_eq!(suppressed.kind, HGroupClueKind::Unrecognized);
            assert!(suppressed.play_identities.is_empty());
            assert_eq!(suppressed.save_identities, three);
            assert_eq!(suppressed.suppression, Some(suppression));
        }
    }

    #[test]
    fn primary_resolution_broad_rank_touch_requires_an_actual_rank_save_identity() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Pink6Suits);
        let three = physical_identity(&variant, 5, 3);
        for rank in [2, 5] {
            let clue = GenericClue::Rank(hanabi_core::ClueRank::new(rank));
            assert_eq!(three.filter_clue(&variant, clue, true), three);
            let mut discarded = [0; 30];
            discarded[three.iter_ids().next().unwrap().index()] = 1;
            let save = ClueInterpretationPlan::direct_save(SaveClueInputs {
                clue,
                variant: &variant,
                focus_identities: three,
                focus_was_chop: true,
                stack_heights: &[0; 6],
                discarded_counts: &discarded,
                two_save_visible_other_counts: &[0; 30],
                two_save_visible_other_chop_counts: &[0; 30],
                critical_visible_other_counts: &[0; 30],
            });
            assert_eq!(save.kind, HGroupClueKind::Save(HGroupSaveKind::Critical));
            let plan = ClueInterpretationPlan::resolve(PrimaryClueInputs {
                clue,
                variant: &variant,
                play_identities: three,
                save_identities: save.save_identities,
                stack_heights: &[0; 6],
                eight_clue_save: false,
                suppressions: [None; 7],
            });
            assert_eq!(plan.kind, save.kind);
            assert_eq!(plan.save_identities, three);
            assert!(plan.play_identities.is_empty());
        }
    }

    #[test]
    fn direct_save_no_variant_base_rules_and_precedence() {
        let variant = VariantSpec::no_variant();
        let color = GenericClue::Color(hanabi_core::ClueColorId::new(0));
        let rank_clue = |rank| GenericClue::Rank(hanabi_core::ClueRank::new(rank));
        // rank, clue, stack height, discards, visible others, visible on chop,
        // focus on chop, expected base Save kind.
        let cases = [
            (
                5,
                rank_clue(5),
                0,
                0,
                0,
                0,
                true,
                Some(HGroupSaveKind::Five),
            ),
            (
                5,
                rank_clue(5),
                4,
                0,
                0,
                0,
                true,
                Some(HGroupSaveKind::Five),
            ),
            (2, rank_clue(2), 0, 0, 0, 0, true, Some(HGroupSaveKind::Two)),
            (2, rank_clue(2), 0, 1, 0, 0, true, Some(HGroupSaveKind::Two)),
            (2, rank_clue(2), 1, 1, 0, 0, true, Some(HGroupSaveKind::Two)),
            (2, rank_clue(2), 0, 0, 1, 0, true, None),
            (2, rank_clue(2), 0, 0, 1, 1, true, Some(HGroupSaveKind::Two)),
            (3, color, 0, 1, 0, 0, true, Some(HGroupSaveKind::Critical)),
            (
                3,
                rank_clue(3),
                0,
                1,
                0,
                0,
                true,
                Some(HGroupSaveKind::Critical),
            ),
            (3, color, 2, 1, 0, 0, true, None),
            (3, color, 0, 0, 0, 0, true, None),
            (3, color, 0, 2, 0, 0, true, None),
            (3, color, 0, 1, 1, 1, true, None),
            (5, color, 0, 0, 0, 0, true, None),
            (2, color, 0, 1, 0, 0, true, Some(HGroupSaveKind::Critical)),
            (5, rank_clue(5), 5, 0, 0, 0, true, None),
            (2, rank_clue(2), 2, 0, 0, 0, true, None),
            (3, color, 3, 1, 0, 0, true, None),
            (5, rank_clue(5), 0, 0, 0, 0, false, None),
            (2, rank_clue(2), 0, 0, 0, 0, false, None),
            (3, color, 0, 1, 0, 0, false, None),
            (3, rank_clue(2), 0, 1, 0, 0, true, None),
        ];
        for (rank, clue, height, discards, visible, chop_visible, on_chop, expected) in cases {
            let focus = physical_identity(&variant, 0, rank);
            let id = focus.iter_ids().next().unwrap();
            let mut discarded_counts = [0; 25];
            let mut two_save_visible_other_counts = [0; 25];
            let mut two_save_visible_other_chop_counts = [0; 25];
            discarded_counts[id.index()] = discards;
            two_save_visible_other_counts[id.index()] = visible;
            two_save_visible_other_chop_counts[id.index()] = chop_visible;
            let plan = ClueInterpretationPlan::direct_save(SaveClueInputs {
                clue,
                variant: &variant,
                focus_identities: focus,
                focus_was_chop: on_chop,
                stack_heights: &[height, 0, 0, 0, 0],
                discarded_counts: &discarded_counts,
                two_save_visible_other_counts: &two_save_visible_other_counts,
                two_save_visible_other_chop_counts: &two_save_visible_other_chop_counts,
                critical_visible_other_counts: &two_save_visible_other_counts,
            });
            assert_eq!(
                plan.kind,
                expected.map_or(HGroupClueKind::Unrecognized, HGroupClueKind::Save),
                "{rank} {clue:?}, height={height}, discards={discards}, visible={visible}, chop_visible={chop_visible}, on_chop={on_chop}"
            );
            assert_eq!(
                plan.save_identities,
                expected.map_or(IdentitySet::default(), |_| focus)
            );
            assert!(plan.play_identities.is_empty());
            assert_eq!(plan.suppression, None);
        }
    }

    #[test]
    fn direct_save_black_sixth_suit_uses_variant_multiplicity() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Black6Suits);
        let focus = physical_identity(&variant, 5, 3);
        let id = focus.iter_ids().next().unwrap();
        assert!(id.index() >= 25);
        assert_eq!(variant.copies(id), Some(1));
        let plan = ClueInterpretationPlan::direct_save(SaveClueInputs {
            clue: GenericClue::Rank(hanabi_core::ClueRank::new(3)),
            variant: &variant,
            focus_identities: focus,
            focus_was_chop: true,
            stack_heights: &[0; 6],
            discarded_counts: &[0; 30],
            two_save_visible_other_counts: &[0; 30],
            two_save_visible_other_chop_counts: &[0; 30],
            critical_visible_other_counts: &[0; 30],
        });
        assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Critical));
        assert_eq!(plan.save_identities, focus);
        assert!(plan.play_identities.is_empty());
        assert_eq!(plan.suppression, None);
    }

    #[test]
    fn direct_save_rainbow_critical_three_uses_every_physical_color_touch() {
        let variant = VariantSpec::from_catalog(hanabi_core::VariantId::Rainbow6Suits);
        let rainbow = physical_identity(&variant, 5, 3);
        let red = physical_identity(&variant, 0, 3);
        let blue = physical_identity(&variant, 3, 3);
        let mut discarded_counts = [0; 30];
        for id in rainbow.union(red).union(blue).iter_ids() {
            discarded_counts[id.index()] = 1;
        }
        for color in variant.clue_colors() {
            let clue = GenericClue::Color(*color);
            assert!(variant.touches(rainbow.iter_ids().next().unwrap(), clue));
            let plan = ClueInterpretationPlan::direct_save(SaveClueInputs {
                clue,
                variant: &variant,
                focus_identities: rainbow.union(red).union(blue),
                focus_was_chop: true,
                stack_heights: &[0; 6],
                discarded_counts: &discarded_counts,
                two_save_visible_other_counts: &[0; 30],
                two_save_visible_other_chop_counts: &[0; 30],
                critical_visible_other_counts: &[0; 30],
            });
            let expected = match color.value() {
                0 => rainbow.union(red),
                3 => rainbow.union(blue),
                _ => rainbow,
            };
            assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Critical));
            assert_eq!(plan.save_identities, expected);
            assert!(plan.play_identities.is_empty());
            assert_eq!(plan.suppression, None);
        }
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
            stack_heights: &[4; 5],
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
            stack_heights: &[1, 0, 0, 0, 0],
            eight_clue_save: false,
            suppressions: [None; 7],
        });
        assert_eq!(plan.kind, HGroupClueKind::Save(HGroupSaveKind::Two));
        assert!(plan.play_identities.is_empty());
    }
}
