use core::fmt;

use hanabi_core::{
    Card, CardId, ClueFacts, FullState, GenericClue, IdentityId, IdentityUniverse, PlayerView,
    VariantSpec, WorldConstructionError, WorldTemplate,
};

type Counts = Vec<u8>;

/// The hidden worlds logically consistent with one [`PlayerView`].
///
/// Direct clues and public card counts define the logical base; optional
/// convention constraints can further restrict the worlds visited by a plan.
#[derive(Debug)]
pub struct InformationSet {
    deductions: LogicalDeductions,
    known_identities: Vec<Option<Card>>,
    constraint_masks: Vec<u64>,
    deck_cards: Vec<CardId>,
    world_template: WorldTemplate,
}

/// Certainties derived from current public state, direct clues, and card counts.
///
/// This contains no convention interpretation and no world construction-only deck
/// reconstruction data, making it suitable for the policy hot path.
#[derive(Debug)]
pub struct LogicalDeductions {
    view: PlayerView,
    unknown_hand_cards: Vec<CardId>,
    possibilities: Vec<(CardId, IdentitySet)>,
    remaining_counts: Counts,
}

/// Compact set whose bit positions are [`IdentityId`] values in an active
/// [`IdentityUniverse`]. Sets may only be combined within the same universe.
/// The card-based `all`, `singleton`, `contains`, and `iter` helpers retain
/// No Variant compatibility.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentitySet(u64);

/// Why deterministic own-hand assignment traversal ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HandAssignmentVisitEnd {
    Exhausted,
    LimitReached,
    VisitorStopped,
}

/// Result of deterministically visiting complete, card-count-consistent own
/// hand assignments for nested perspective analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HandAssignmentVisit {
    examined: usize,
    pub(crate) end: HandAssignmentVisitEnd,
}

/// Bounded count of complete identity assignments.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldCount {
    Exact(u64),
    /// At least this many worlds exist; counting stopped at the configured
    /// bound.
    LowerBound(u64),
}

impl WorldCount {
    #[must_use]
    pub const fn worlds(self) -> u64 {
        match self {
            Self::Exact(worlds) | Self::LowerBound(worlds) => worlds,
        }
    }

    #[must_use]
    pub const fn is_exact(self) -> bool {
        matches!(self, Self::Exact(_))
    }
}

/// Convention-derived restrictions on the root belief state.
///
/// `constraints` always apply. Each factor is a disjunction of mutually
/// exclusive conjunctions. All factors must hold. No factors means only the
/// common constraints apply; an empty factor is a contradiction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BeliefConstraints {
    pub constraints: Vec<(CardId, IdentitySet)>,
    pub factors: Vec<Vec<Vec<(CardId, IdentitySet)>>>,
}

impl IdentitySet {
    const STANDARD_MASK: u64 = (1_u64 << 25) - 1;

    pub(crate) const fn from_mask(mask: u32) -> Self {
        Self((mask as u64) & Self::STANDARD_MASK)
    }

    pub(crate) const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// All No Variant identities, in legacy standard order.
    #[must_use]
    pub const fn all() -> Self {
        Self(Self::STANDARD_MASK)
    }

    /// A No Variant identity, in legacy standard order.
    #[must_use]
    pub const fn singleton(card: Card) -> Self {
        Self(1_u64 << card.index())
    }

    /// Tests a No Variant identity, in legacy standard order.
    #[must_use]
    pub const fn contains(self, card: Card) -> bool {
        self.0 & (1_u64 << card.index()) != 0
    }

    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    /// Iterates No Variant cards. Generic callers must use [`Self::iter_in`].
    pub fn iter(self) -> impl Iterator<Item = Card> {
        self.iter_in(no_variant_universe())
    }

    #[must_use]
    pub fn all_in(universe: &IdentityUniverse) -> Self {
        universe
            .identities()
            .iter()
            .fold(Self::default(), |set, identity| {
                set.union(Self::singleton_id(identity.id))
            })
    }

    /// IDs outside the 64-bit storage domain produce an empty set.
    #[must_use]
    pub const fn singleton_id(identity: IdentityId) -> Self {
        match 1_u64.checked_shl(identity.value() as u32) {
            Some(bit) => Self(bit),
            None => Self(0),
        }
    }

    #[must_use]
    pub const fn contains_id(self, identity: IdentityId) -> bool {
        !self.intersection(Self::singleton_id(identity)).is_empty()
    }

    pub fn iter_ids(self) -> impl Iterator<Item = IdentityId> {
        IdentitySetIter(self.0)
    }

    pub fn iter_in(self, universe: &IdentityUniverse) -> impl Iterator<Item = Card> + '_ {
        self.iter_ids()
            .filter_map(move |id| universe.get(id))
            .map(|identity| Card::new(identity.suit, identity.rank))
    }

    /// Direct positive/negative physical clue filtering, without conventions.
    #[must_use]
    pub fn filter_clue(self, variant: &VariantSpec, clue: GenericClue, touched: bool) -> Self {
        variant
            .identities()
            .identities()
            .iter()
            .filter(|identity| {
                self.contains_id(identity.id) && variant.touches(identity.id, clue) == touched
            })
            .fold(Self::default(), |set, identity| {
                set.union(Self::singleton_id(identity.id))
            })
    }
}

fn no_variant_universe() -> &'static IdentityUniverse {
    static VARIANT: std::sync::OnceLock<VariantSpec> = std::sync::OnceLock::new();
    VARIANT.get_or_init(VariantSpec::no_variant).identities()
}

struct IdentitySetIter(u64);

impl Iterator for IdentitySetIter {
    type Item = IdentityId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let index = u8::try_from(self.0.trailing_zeros()).ok()?;
        self.0 &= self.0 - 1;
        Some(IdentityId::new(index))
    }
}

impl Clone for InformationSet {
    fn clone(&self) -> Self {
        Self {
            deductions: self.deductions.clone(),
            known_identities: self.known_identities.clone(),
            constraint_masks: self.constraint_masks.clone(),
            deck_cards: self.deck_cards.clone(),
            world_template: self.world_template.clone(),
        }
    }
}

impl PartialEq for InformationSet {
    fn eq(&self, other: &Self) -> bool {
        self.deductions == other.deductions
            && self.known_identities == other.known_identities
            && self.constraint_masks == other.constraint_masks
            && self.deck_cards == other.deck_cards
    }
}

impl Eq for InformationSet {}

impl LogicalDeductions {
    /// Derives exact possible identities without constructing complete worlds.
    ///
    /// # Errors
    ///
    /// Returns [`InformationSetError`] when visible card counts or direct clues
    /// are inconsistent.
    pub fn new(view: PlayerView) -> Result<Self, InformationSetError> {
        let own_hand = view
            .hands
            .get(view.observer.index())
            .ok_or(InformationSetError::InvalidObserver)?;
        let unknown_hand_cards = own_hand.iter().map(|card| card.id).collect::<Vec<_>>();
        let constraints = own_hand.iter().map(|card| card.clues).collect::<Vec<_>>();
        let universe = view.variant.identities();
        let mut remaining_counts = universe
            .identities()
            .iter()
            .map(|identity| identity.copies)
            .collect::<Counts>();
        for identity in view
            .hands
            .iter()
            .flatten()
            .filter_map(|card| card.identity)
            .chain(
                view.play_stacks
                    .iter()
                    .flatten()
                    .map(|(_, identity)| *identity),
            )
            .chain(view.discard_pile.iter().map(|(_, identity)| *identity))
        {
            let id = universe
                .id_for(identity.suit, identity.rank)
                .ok_or(InformationSetError::UnknownIdentity(identity))?;
            let count = &mut remaining_counts[id.index()];
            if *count == 0 {
                return Err(InformationSetError::TooManyVisibleCopies(identity));
            }
            *count -= 1;
        }
        let feasible_by_slot = feasible_identities(&constraints, &remaining_counts, &view.variant);
        if feasible_by_slot
            .iter()
            .any(|identities| identities.is_empty())
        {
            return Err(InformationSetError::NoConsistentWorld);
        }
        let possibilities = unknown_hand_cards
            .iter()
            .copied()
            .zip(feasible_by_slot)
            .collect();
        Ok(Self {
            view,
            unknown_hand_cards,
            possibilities,
            remaining_counts,
        })
    }

    #[must_use]
    pub const fn view(&self) -> &PlayerView {
        &self.view
    }

    #[must_use]
    pub fn unknown_hand_cards(&self) -> &[CardId] {
        &self.unknown_hand_cards
    }

    #[must_use]
    pub fn possible_identities(&self, card: CardId) -> Option<IdentitySet> {
        self.possibilities
            .iter()
            .find_map(|(candidate, identities)| (*candidate == card).then_some(*identities))
    }

    /// Visits complete own-hand assignments without inventing impossible card
    /// combinations. Returning `true` from `visitor` stops traversal.
    pub(crate) fn visit_hand_assignments(
        &self,
        limit: usize,
        mut visitor: impl FnMut(&[(CardId, Card)]) -> bool,
    ) -> HandAssignmentVisit {
        let mut remaining = self.remaining_counts.clone();
        let mut assignment = Vec::with_capacity(self.unknown_hand_cards.len());
        let mut result = HandAssignmentVisit {
            examined: 0,
            end: HandAssignmentVisitEnd::Exhausted,
        };
        visit_assignments(
            self.view.variant.identities(),
            &self.unknown_hand_cards,
            &self.possibilities,
            0,
            &mut remaining,
            &mut assignment,
            limit,
            &mut visitor,
            &mut result,
        );
        result
    }
}

#[allow(clippy::too_many_arguments)]
fn visit_assignments(
    universe: &IdentityUniverse,
    cards: &[CardId],
    possibilities: &[(CardId, IdentitySet)],
    slot: usize,
    remaining: &mut Counts,
    assignment: &mut Vec<(CardId, Card)>,
    limit: usize,
    visitor: &mut impl FnMut(&[(CardId, Card)]) -> bool,
    result: &mut HandAssignmentVisit,
) {
    if result.end != HandAssignmentVisitEnd::Exhausted {
        return;
    }
    if slot == cards.len() {
        if result.examined == limit {
            result.end = HandAssignmentVisitEnd::LimitReached;
            return;
        }
        result.examined += 1;
        if visitor(assignment) {
            result.end = HandAssignmentVisitEnd::VisitorStopped;
        }
        return;
    }
    let card = cards[slot];
    let identities = possibilities
        .iter()
        .find_map(|(candidate, identities)| (*candidate == card).then_some(*identities))
        .unwrap_or_default();
    for id in identities.iter_ids() {
        let Some(identity) = universe.get(id) else {
            continue;
        };
        let count = &mut remaining[id.index()];
        if *count == 0 {
            continue;
        }
        *count -= 1;
        assignment.push((card, Card::new(identity.suit, identity.rank)));
        visit_assignments(
            universe,
            cards,
            possibilities,
            slot + 1,
            remaining,
            assignment,
            limit,
            visitor,
            result,
        );
        assignment.pop();
        remaining[id.index()] += 1;
        if result.end != HandAssignmentVisitEnd::Exhausted {
            return;
        }
    }
}

impl Clone for LogicalDeductions {
    fn clone(&self) -> Self {
        Self {
            view: self.view.clone(),
            unknown_hand_cards: self.unknown_hand_cards.clone(),
            possibilities: self.possibilities.clone(),
            remaining_counts: self.remaining_counts.clone(),
        }
    }
}

impl PartialEq for LogicalDeductions {
    fn eq(&self, other: &Self) -> bool {
        self.view == other.view
            && self.unknown_hand_cards == other.unknown_hand_cards
            && self.possibilities == other.possibilities
            && self.remaining_counts == other.remaining_counts
    }
}

impl Eq for LogicalDeductions {}

impl InformationSet {
    /// Builds card-count and direct-clue constraints from a legal observation.
    ///
    /// # Errors
    ///
    /// Returns [`InformationSetError`] when card locations or visible
    /// identities are inconsistent, or when no hidden world satisfies all
    /// direct clues.
    pub fn new(view: &PlayerView) -> Result<Self, InformationSetError> {
        let deductions = LogicalDeductions::new(view.clone())?;

        let card_count = view.hands.iter().map(Vec::len).sum::<usize>()
            + view.play_stacks.iter().map(Vec::len).sum::<usize>()
            + view.discard_pile.len()
            + view.deck_size;
        let mut occupied = vec![false; card_count];
        let mut known_identities = vec![None; card_count];
        let mut constraint_masks = Vec::new();

        for hand in &view.hands {
            for observed in hand {
                mark_location(&mut occupied, observed.id)?;
                if let Some(identity) = observed.identity {
                    set_known_identity(&mut known_identities, observed.id, identity)?;
                } else {
                    constraint_masks.push(observed.clues.identity_mask_in(&view.variant));
                }
            }
        }
        for stack in &view.play_stacks {
            for (card, identity) in stack {
                mark_location(&mut occupied, *card)?;
                set_known_identity(&mut known_identities, *card, *identity)?;
            }
        }
        for (card, identity) in &view.discard_pile {
            mark_location(&mut occupied, *card)?;
            set_known_identity(&mut known_identities, *card, *identity)?;
        }

        let deck_cards = occupied
            .iter()
            .enumerate()
            .filter_map(|(index, is_occupied)| (!is_occupied).then_some(CardId::new(index)))
            .collect::<Vec<_>>();
        if deck_cards.len() != view.deck_size {
            return Err(InformationSetError::DeckSizeMismatch {
                observed: view.deck_size,
                inferred: deck_cards.len(),
            });
        }

        let remaining_counts = deductions.remaining_counts.clone();

        let remaining_total: usize = remaining_counts
            .iter()
            .map(|count| usize::from(*count))
            .sum();
        let hidden_total = deductions.unknown_hand_cards.len() + deck_cards.len();
        if remaining_total != hidden_total {
            return Err(InformationSetError::HiddenCardCountMismatch {
                identities: remaining_total,
                locations: hidden_total,
            });
        }

        let world_template =
            WorldTemplate::new(view).map_err(InformationSetError::InvalidWorldTemplate)?;

        Ok(Self {
            deductions,
            known_identities,
            constraint_masks,
            deck_cards,
            world_template,
        })
    }

    #[must_use]
    pub const fn view(&self) -> &PlayerView {
        self.deductions.view()
    }

    /// Convention-agnostic logical analysis shared with symbolic planning.
    #[must_use]
    pub const fn deductions(&self) -> &LogicalDeductions {
        &self.deductions
    }

    #[must_use]
    pub fn possible_identities(&self, card: CardId) -> Option<IdentitySet> {
        self.deductions.possible_identities(card)
    }

    /// Counts complete, identity-distinct hidden worlds admitted by direct
    /// clues and optional convention constraints.
    ///
    /// Counting stops as soon as `limit + 1` worlds have been found. This is
    /// the inexpensive gate used by exact endgame planning; it avoids trying
    /// to represent the astronomical opening belief space.
    ///
    /// # Panics
    /// Panics only if the internally created unlimited control is interrupted.
    #[must_use]
    pub fn world_count_up_to(&self, belief: &BeliefConstraints, limit: u64) -> WorldCount {
        self.world_count_with_control(belief, limit, &crate::AnalysisControl::default())
            .expect("unlimited world counting cannot be interrupted")
    }

    pub(crate) fn world_count_with_control(
        &self,
        belief: &BeliefConstraints,
        limit: u64,
        control: &crate::AnalysisControl,
    ) -> Result<WorldCount, crate::AnalysisStopped> {
        let stop_after = limit.saturating_add(1);
        let mut worlds = 0_u64;
        let mut stopped = None;
        self.visit_belief_masks(belief, control, |masks| {
            let mut counts = self.deductions.remaining_counts.clone();
            stopped = count_identity_worlds(
                masks,
                self.deck_cards.len(),
                0,
                &mut counts,
                stop_after,
                &mut worlds,
                control,
            )
            .err();
            stopped.is_none() && worlds < stop_after
        })?;
        if let Some(error) = stopped {
            return Err(error);
        }
        if worlds <= limit {
            Ok(WorldCount::Exact(worlds))
        } else {
            Ok(WorldCount::LowerBound(stop_after))
        }
    }

    /// Visits every complete hidden world admitted by direct clues and
    /// convention constraints, provided the set contains at most `limit`
    /// worlds.
    ///
    /// # Errors
    ///
    /// Returns [`EnumerateWorldsError::LimitExceeded`] before invoking the
    /// visitor when the exact-planning gate is too large, or wraps an invalid
    /// world construction if an internal invariant is violated.
    pub fn visit_worlds(
        &self,
        belief: &BeliefConstraints,
        limit: u64,
        visitor: impl FnMut(FullState),
    ) -> Result<u64, EnumerateWorldsError> {
        let count = self.world_count_up_to(belief, limit);
        let WorldCount::Exact(_) = count else {
            return Err(EnumerateWorldsError::LimitExceeded {
                limit,
                at_least: count.worlds(),
            });
        };

        self.visit_worlds_after_count(belief, &crate::AnalysisControl::default(), visitor)
    }

    /// Materializes worlds after the caller has already completed the bounded
    /// count used by the exact-planning gate.
    pub(crate) fn collect_worlds_after_count(
        &self,
        belief: &BeliefConstraints,
        capacity: usize,
        control: &crate::AnalysisControl,
    ) -> Result<Vec<FullState>, EnumerateWorldsError> {
        let mut worlds = Vec::with_capacity(capacity);
        self.visit_worlds_after_count(belief, control, |world| worlds.push(world))?;
        Ok(worlds)
    }

    fn visit_worlds_after_count(
        &self,
        belief: &BeliefConstraints,
        control: &crate::AnalysisControl,
        mut visitor: impl FnMut(FullState),
    ) -> Result<u64, EnumerateWorldsError> {
        let universe = self.view().variant.identities();
        let identity = &universe.identities()[0];
        let placeholder = Card::new(identity.suit, identity.rank);
        let base_cards = self
            .known_identities
            .iter()
            .map(|identity| identity.unwrap_or(placeholder))
            .collect::<Vec<_>>();
        let mut visited = 0_u64;
        let mut error = None;
        self.visit_belief_masks(belief, control, |masks| {
            let mut locations = self.deductions.unknown_hand_cards.clone();
            locations.extend(self.deck_cards.iter().copied());
            let mut cards = base_cards.clone();
            let mut counts = self.deductions.remaining_counts.clone();
            let finished = visit_identity_worlds(
                universe,
                &locations,
                masks,
                0,
                &mut counts,
                &mut cards,
                &mut |cards| match self.world_template.instantiate(cards.to_vec()) {
                    Ok(state) => {
                        visited += 1;
                        visitor(state);
                        true
                    }
                    Err(source) => {
                        error = Some(source);
                        false
                    }
                },
                control,
            );
            finished && error.is_none()
        })
        .map_err(EnumerateWorldsError::Stopped)?;
        control
            .checkpoint()
            .map_err(EnumerateWorldsError::Stopped)?;
        if let Some(source) = error {
            return Err(EnumerateWorldsError::WorldConstruction(source));
        }
        Ok(visited)
    }

    fn visit_belief_masks(
        &self,
        belief: &BeliefConstraints,
        control: &crate::AnalysisControl,
        mut visitor: impl FnMut(&[u64]) -> bool,
    ) -> Result<(), crate::AnalysisStopped> {
        let base_masks = self.constrained_masks(&belief.constraints);
        if base_masks.contains(&0) {
            return Ok(());
        }
        self.visit_factors(&belief.factors, &base_masks, &mut visitor, control)?;
        Ok(())
    }

    /// Descend only as far as the bounded world visitor requests. Intersecting
    /// at each level also prunes contradictions before expanding later factors.
    fn visit_factors(
        &self,
        factors: &[Vec<Vec<(CardId, IdentitySet)>>],
        masks: &[u64],
        visitor: &mut impl FnMut(&[u64]) -> bool,
        control: &crate::AnalysisControl,
    ) -> Result<bool, crate::AnalysisStopped> {
        control.checkpoint()?;
        let Some((factor, remaining)) = factors.split_first() else {
            return Ok(visitor(masks));
        };
        for alternative in factor {
            let mut next = masks.to_vec();
            self.intersect_masks(&mut next, alternative);
            if !next.contains(&0) && !self.visit_factors(remaining, &next, visitor, control)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn constrained_masks(&self, constraints: &[(CardId, IdentitySet)]) -> Vec<u64> {
        let mut masks = self.constraint_masks.clone();
        self.intersect_masks(&mut masks, constraints);
        masks
    }

    fn intersect_masks(&self, masks: &mut [u64], constraints: &[(CardId, IdentitySet)]) {
        for (card, identities) in constraints {
            let Some(slot) = self
                .deductions
                .unknown_hand_cards
                .iter()
                .position(|candidate| candidate == card)
            else {
                continue;
            };
            masks[slot] &= identities.0;
        }
    }
}

fn count_identity_worlds(
    hand_masks: &[u64],
    deck_len: usize,
    slot: usize,
    counts: &mut Counts,
    stop_after: u64,
    worlds: &mut u64,
    control: &crate::AnalysisControl,
) -> Result<(), crate::AnalysisStopped> {
    control.checkpoint()?;
    if *worlds >= stop_after {
        return Ok(());
    }
    let location_count = hand_masks.len() + deck_len;
    if slot == location_count {
        *worlds += 1;
        return Ok(());
    }
    let allowed = hand_masks.get(slot).copied().unwrap_or(u64::MAX);
    for identity in 0..counts.len() {
        if counts[identity] == 0 || allowed & (1_u64 << identity) == 0 {
            continue;
        }
        counts[identity] -= 1;
        let result = count_identity_worlds(
            hand_masks,
            deck_len,
            slot + 1,
            counts,
            stop_after,
            worlds,
            control,
        );
        counts[identity] += 1;
        result?;
        if *worlds >= stop_after {
            return Ok(());
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn visit_identity_worlds(
    universe: &IdentityUniverse,
    locations: &[CardId],
    hand_masks: &[u64],
    slot: usize,
    counts: &mut Counts,
    cards: &mut [Card],
    visitor: &mut impl FnMut(&[Card]) -> bool,
    control: &crate::AnalysisControl,
) -> bool {
    if control.checkpoint().is_err() {
        return false;
    }
    if slot == locations.len() {
        return visitor(cards);
    }
    let allowed = hand_masks.get(slot).copied().unwrap_or(u64::MAX);
    for spec in universe.identities() {
        let identity = spec.id.index();
        if counts[identity] == 0 || allowed & (1_u64 << identity) == 0 {
            continue;
        }
        counts[identity] -= 1;
        cards[locations[slot].index()] = Card::new(spec.suit, spec.rank);
        if !visit_identity_worlds(
            universe,
            locations,
            hand_masks,
            slot + 1,
            counts,
            cards,
            visitor,
            control,
        ) {
            counts[identity] += 1;
            return false;
        }
        counts[identity] += 1;
    }
    true
}

fn mark_location(occupied: &mut [bool], card: CardId) -> Result<(), InformationSetError> {
    let Some(slot) = occupied.get_mut(card.index()) else {
        return Err(InformationSetError::InvalidCardId(card));
    };
    if *slot {
        return Err(InformationSetError::DuplicateLocation(card));
    }
    *slot = true;
    Ok(())
}

fn set_known_identity(
    identities: &mut [Option<Card>],
    card: CardId,
    identity: Card,
) -> Result<(), InformationSetError> {
    let Some(slot) = identities.get_mut(card.index()) else {
        return Err(InformationSetError::InvalidCardId(card));
    };
    if let Some(existing) = slot {
        if *existing != identity {
            return Err(InformationSetError::ConflictingIdentity {
                card,
                first: *existing,
                second: identity,
            });
        }
    } else {
        *slot = Some(identity);
    }
    Ok(())
}

fn feasible_identities(
    constraints: &[ClueFacts],
    counts: &[u8],
    variant: &VariantSpec,
) -> Vec<IdentitySet> {
    feasible_identity_array(constraints, counts, variant)[..constraints.len()].to_vec()
}

fn feasible_identity_array(
    constraints: &[ClueFacts],
    counts: &[u8],
    variant: &VariantSpec,
) -> [IdentitySet; 5] {
    debug_assert!(constraints.len() <= 5);
    let mut masks = [0_u64; 5];
    for (slot, constraint) in constraints.iter().enumerate() {
        masks[slot] = constraint.identity_mask_in(variant);
    }
    feasible_identity_masks(&masks[..constraints.len()], counts)
}

fn feasible_identity_masks(constraint_masks: &[u64], counts: &[u8]) -> [IdentitySet; 5] {
    debug_assert!(constraint_masks.len() <= 5);
    let mut allowed = [0_u64; 5];
    allowed[..constraint_masks.len()].copy_from_slice(constraint_masks);
    let subset_count = 1_usize << constraint_masks.len();
    let mut union_masks = [0_u64; 32];
    let mut union_capacities = [0_usize; 32];
    for subset in 1..subset_count {
        let slot = subset.trailing_zeros() as usize;
        let previous = subset & !(1 << slot);
        let union = union_masks[previous] | allowed[slot];
        union_masks[subset] = union;
        let mut capacity = union_capacities[previous];
        let mut identities = allowed[slot] & !union_masks[previous];
        while identities != 0 {
            let identity = identities.trailing_zeros() as usize;
            identities &= identities - 1;
            capacity += usize::from(counts[identity]);
        }
        union_capacities[subset] = capacity;
    }

    let mut feasible = [IdentitySet::default(); 5];
    let available_identities = counts
        .iter()
        .enumerate()
        .fold(0_u64, |mask, (identity, count)| {
            mask | (u64::from(*count > 0) << identity)
        });
    for fixed_slot in 0..constraint_masks.len() {
        let mut forbidden = 0_u64;
        let mut impossible = false;
        for subset in (1..subset_count).filter(|subset| subset & (1 << fixed_slot) == 0) {
            let required = subset.count_ones() as usize;
            match union_capacities[subset].cmp(&required) {
                core::cmp::Ordering::Less => {
                    impossible = true;
                    break;
                }
                core::cmp::Ordering::Equal => forbidden |= union_masks[subset],
                core::cmp::Ordering::Greater => {}
            }
        }
        if !impossible {
            feasible[fixed_slot] =
                IdentitySet::from_bits(allowed[fixed_slot] & available_identities & !forbidden);
        }
    }
    feasible
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InformationSetError {
    UnknownIdentity(Card),
    InvalidObserver,
    InvalidCardId(CardId),
    DuplicateLocation(CardId),
    ConflictingIdentity {
        card: CardId,
        first: Card,
        second: Card,
    },
    TooManyVisibleCopies(Card),
    DeckSizeMismatch {
        observed: usize,
        inferred: usize,
    },
    HiddenCardCountMismatch {
        identities: usize,
        locations: usize,
    },
    InvalidWorldTemplate(WorldConstructionError),
    NoConsistentWorld,
}

impl fmt::Display for InformationSetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIdentity(card) => {
                write!(formatter, "identity {card} is not declared by the variant")
            }
            Self::InvalidObserver => formatter.write_str("observer does not have a hand"),
            Self::InvalidCardId(card) => write!(formatter, "invalid card identifier {card}"),
            Self::DuplicateLocation(card) => {
                write!(formatter, "card {card} occurs in multiple public locations")
            }
            Self::ConflictingIdentity {
                card,
                first,
                second,
            } => write!(
                formatter,
                "card {card} has conflicting visible identities {first} and {second}"
            ),
            Self::TooManyVisibleCopies(card) => {
                write!(formatter, "too many visible copies of {card}")
            }
            Self::DeckSizeMismatch { observed, inferred } => write!(
                formatter,
                "observed deck has {observed} cards but card locations imply {inferred}"
            ),
            Self::HiddenCardCountMismatch {
                identities,
                locations,
            } => write!(
                formatter,
                "{identities} hidden identities remain for {locations} hidden locations"
            ),
            Self::InvalidWorldTemplate(error) => {
                write!(
                    formatter,
                    "invalid public state for exact planning: {error}"
                )
            }
            Self::NoConsistentWorld => {
                formatter.write_str("no hidden world satisfies all direct clues")
            }
        }
    }
}

impl std::error::Error for InformationSetError {}

/// Failure while exhaustively materializing a bounded belief state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnumerateWorldsError {
    Stopped(crate::AnalysisStopped),
    LimitExceeded { limit: u64, at_least: u64 },
    WorldConstruction(WorldConstructionError),
}

impl fmt::Display for EnumerateWorldsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped(error) => error.fmt(formatter),
            Self::LimitExceeded { limit, at_least } => write!(
                formatter,
                "belief contains at least {at_least} worlds, exceeding exact limit {limit}"
            ),
            Self::WorldConstruction(error) => {
                write!(formatter, "invalid enumerated world: {error}")
            }
        }
    }
}

impl std::error::Error for EnumerateWorldsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Stopped(error) => Some(error),
            Self::LimitExceeded { .. } => None,
            Self::WorldConstruction(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use hanabi_core::{FullState, PlayerId, Rank, Suit, standard_deck};

    #[test]
    fn widened_identity_set_preserves_standard_bit_order() {
        let last = Card::new(Suit::Purple, Rank::Five);
        let singleton = IdentitySet::singleton(last);

        assert_eq!(IdentitySet::all().0, (1_u64 << 25) - 1);
        assert_eq!(singleton.0, 1_u64 << 24);
        assert!(singleton.contains(last));
        assert_eq!(singleton.len(), 1);
    }

    #[test]
    fn bounded_world_count_short_circuits_openings() {
        let state = FullState::new_standard(2, standard_deck()).unwrap();
        let information = InformationSet::new(&state.view_for(PlayerId::new(0)).unwrap()).unwrap();
        assert_eq!(
            information.world_count_up_to(&BeliefConstraints::default(), 32),
            WorldCount::LowerBound(33)
        );
    }

    #[test]
    fn lazy_factors_stop_without_expansion_and_preserve_contradictions() {
        let state = FullState::new_standard(2, standard_deck()).unwrap();
        let information = InformationSet::new(&state.view_for(PlayerId::new(0)).unwrap()).unwrap();
        let first_card = information.deductions.unknown_hand_cards[0];
        let red = IdentitySet::singleton(Card::new(Suit::Red, Rank::One));
        let factor = vec![
            vec![(first_card, red)],
            vec![(first_card, IdentitySet::all().without(red))],
        ];
        // Forty factors would have required a trillion eager branch entries.
        let belief = BeliefConstraints {
            constraints: Vec::new(),
            factors: vec![factor; 40],
        };
        assert_eq!(
            information.world_count_up_to(&belief, 1),
            WorldCount::LowerBound(2)
        );
        let impossible = BeliefConstraints {
            constraints: Vec::new(),
            factors: vec![Vec::new()],
        };
        assert_eq!(
            information.world_count_up_to(&impossible, 1),
            WorldCount::Exact(0)
        );
        let control = crate::AnalysisControl::new(crate::CancellationToken::default(), None, 2);
        assert_eq!(
            information.world_count_with_control(&belief, 1, &control),
            Err(crate::AnalysisStopped::WorkLimit)
        );
    }

    #[test]
    fn hand_assignment_visit_has_one_exhaustive_stop_reason() {
        let state = FullState::new_standard(2, standard_deck()).unwrap();
        let mut deductions =
            LogicalDeductions::new(state.view_for(PlayerId::new(0)).unwrap()).unwrap();

        let limited = deductions.visit_hand_assignments(0, |_| false);
        assert_eq!(limited.examined, 0);
        assert_eq!(limited.end, HandAssignmentVisitEnd::LimitReached);

        let stopped = deductions.visit_hand_assignments(1, |_| true);
        assert_eq!(stopped.examined, 1);
        assert_eq!(stopped.end, HandAssignmentVisitEnd::VisitorStopped);

        deductions.unknown_hand_cards.clear();
        let exhausted = deductions.visit_hand_assignments(1, |_| false);
        assert_eq!(exhausted.examined, 1);
        assert_eq!(exhausted.end, HandAssignmentVisitEnd::Exhausted);
    }

    #[test]
    fn exact_world_enumeration_is_complete_unique_and_observation_safe() {
        let mut state = FullState::new_standard(2, standard_deck()).unwrap();
        while state.deck_size() > 0 && !state.is_terminal() {
            let playable = state
                .hand(state.current_player())
                .unwrap()
                .iter()
                .find(|card| {
                    let identity = state.card(**card).unwrap();
                    identity.rank.number()
                        == u8::try_from(state.play_stacks()[identity.suit.index()].len()).unwrap()
                            + 1
                });
            let action = playable.map_or_else(
                || {
                    state
                        .legal_actions()
                        .into_iter()
                        .find(|action| matches!(action, hanabi_core::Action::Discard(_)))
                        .unwrap_or_else(|| {
                            state
                                .legal_actions()
                                .into_iter()
                                .find(|action| matches!(action, hanabi_core::Action::Clue { .. }))
                                .unwrap()
                        })
                },
                |card| hanabi_core::Action::Play(*card),
            );
            state.apply(action).unwrap();
        }
        assert!(!state.is_terminal());
        let observer = state.current_player();
        let view = state.view_for(observer).unwrap();
        let information = InformationSet::new(&view.clone()).unwrap();
        let belief = BeliefConstraints::default();
        let count = information.world_count_up_to(&belief, 100_000);
        assert!(count.is_exact());
        assert!(count.worlds() > 1);

        let mut unique = HashSet::new();
        let visited = information
            .visit_worlds(&belief, 100_000, |world| {
                assert_eq!(world.view_for(observer), Some(view.clone()));
                assert!(unique.insert(format!("{world:?}")));
            })
            .unwrap();
        assert_eq!(visited, count.worlds());
        assert_eq!(u64::try_from(unique.len()).unwrap(), count.worlds());
    }
}
