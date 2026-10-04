use core::fmt;

use hanabi_core::{
    Action, CardId, GameStatus, GenericClue, MAX_CLUE_TOKENS, ObservedCard, PhysicalAction,
    PlayerId, PlayerView,
};

use crate::{IdentitySet, LogicalDeductions};

/// What direct clues, public cards, and card-count elimination prove about one
/// card. No meaning is assigned to why any clue was given.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CardAssessment {
    pub certainly_playable: bool,
    pub certainly_useless: bool,
}

/// Assesses every identity that remains logically possible for a hidden card.
///
/// A card is certainly playable only when every possible identity can be
/// played now. It is certainly useless only when every possible identity has
/// already played or can never play because a required preceding rank is exhausted.
/// Returns `None` when `card` is not an unknown card in this information set.
#[must_use]
pub fn assess_card<C: Copy + Into<GenericClue>>(
    deductions: &LogicalDeductions<C>,
    card: CardId,
) -> Option<CardAssessment> {
    let possibilities = deductions.possible_identities(card)?;
    let context = AssessmentContext::new(deductions.view());

    Some(context.assess(possibilities))
}

struct AssessmentContext {
    playable: IdentitySet,
    useless: IdentitySet,
}

impl AssessmentContext {
    fn new<C>(view: &PlayerView<C>) -> Self {
        let variant = &view.variant;
        let universe = variant.identities();
        let stack_heights = view.play_stacks.iter().map(Vec::len).collect::<Vec<_>>();
        let mut discarded_counts = vec![0_usize; universe.identities().len()];
        for (_, card) in &view.discard_pile {
            let id = universe
                .id_for(card.suit, card.rank)
                .expect("deductions validate public identities");
            discarded_counts[id.index()] += 1;
        }
        let mut playable = IdentitySet::default();
        let mut useless = IdentitySet::default();
        for (suit_index, suit) in variant.stack_suits().iter().enumerate() {
            let height = stack_heights[suit_index];
            let mut blocked = false;
            for (rank_index, rank) in variant.card_ranks().iter().enumerate() {
                let Some(id) = universe.id_for(*suit, *rank) else {
                    // A missing identity also makes the remaining path unreachable.
                    if rank_index >= height {
                        blocked = true;
                    }
                    continue;
                };
                let identity = IdentitySet::singleton_id(id);
                if variant.playable(&stack_heights, id) {
                    playable = playable.union(identity);
                }
                if rank_index >= height {
                    blocked |= discarded_counts[id.index()]
                        >= usize::from(variant.copies(id).expect("active identity"));
                }
                if rank_index < height || blocked {
                    useless = useless.union(identity);
                }
            }
        }
        Self { playable, useless }
    }

    fn assess(&self, possibilities: IdentitySet) -> CardAssessment {
        CardAssessment {
            certainly_playable: !possibilities.is_empty()
                && possibilities.without(self.playable).is_empty(),
            certainly_useless: !possibilities.is_empty()
                && possibilities.without(self.useless).is_empty(),
        }
    }
}

/// A deliberately convention-agnostic action policy.
///
/// It never gives a clue or interprets why a clue was given. In order, it:
///
/// 1. plays the oldest certainly playable card;
/// 2. discards the oldest certainly useless card;
/// 3. otherwise discards the oldest card; or
/// 4. when discarding is illegal at full clues, blind-plays the newest card.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConventionAgnosticPolicy;

impl ConventionAgnosticPolicy {
    /// Chooses one action using only direct clues and logical card counts.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the observation is not actionable.
    pub fn select_action(self, deductions: &LogicalDeductions) -> Result<Action, PolicyError> {
        self.select_physical_action(deductions).and_then(|action| {
            Action::try_from(action).map_err(|_| PolicyError::NoConventionAction)
        })
    }

    /// Chooses a physical play or discard using the active variant's rules.
    /// Public generic clue history is consumed directly without legacy conversion.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the observation is not actionable.
    pub fn select_physical_action<C: Copy + Into<GenericClue>>(
        self,
        deductions: &LogicalDeductions<C>,
    ) -> Result<PhysicalAction, PolicyError> {
        let view = deductions.view();
        let hand = view
            .hands
            .get(view.observer.index())
            .ok_or(PolicyError::MissingOwnHand)?;
        let context = AssessmentContext::new(view);
        select_action_from_knowledge(
            view.observer,
            view.current_player,
            view.status,
            view.clue_tokens,
            hand,
            &context,
            |card| deductions.possible_identities(card),
        )
    }
}

fn select_action_from_knowledge(
    observer: PlayerId,
    current_player: PlayerId,
    status: GameStatus,
    clue_tokens: u8,
    hand: &[ObservedCard],
    context: &AssessmentContext,
    possible_identities: impl Fn(CardId) -> Option<IdentitySet>,
) -> Result<PhysicalAction, PolicyError> {
    if status != GameStatus::InProgress || observer != current_player {
        return Err(PolicyError::NotCurrentPlayer);
    }
    if hand.is_empty() {
        return Err(PolicyError::EmptyOwnHand);
    }

    let mut first_useless = None;
    for observed in hand {
        let possibilities = possible_identities(observed.id)
            .ok_or(PolicyError::MissingPossibilities(observed.id))?;
        let assessment = context.assess(possibilities);
        if assessment.certainly_playable {
            return Ok(PhysicalAction::Play(observed.id));
        }
        if first_useless.is_none() && assessment.certainly_useless {
            first_useless = Some(observed.id);
        }
    }

    // Hands are stored oldest-to-newest; a drawn card is appended.
    if clue_tokens < MAX_CLUE_TOKENS {
        return Ok(PhysicalAction::Discard(first_useless.unwrap_or(hand[0].id)));
    }
    Ok(PhysicalAction::Play(
        hand.last().expect("the hand was checked as nonempty").id,
    ))
}

/// Why an action policy could not choose an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    NotCurrentPlayer,
    MissingOwnHand,
    EmptyOwnHand,
    MissingPossibilities(CardId),
    NoConventionAction,
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotCurrentPlayer => formatter.write_str(
                "the information set is not an in-progress position for the current player",
            ),
            Self::MissingOwnHand => formatter.write_str("the observer's hand is missing"),
            Self::EmptyOwnHand => formatter.write_str("the observer's hand is empty"),
            Self::MissingPossibilities(card) => {
                write!(
                    formatter,
                    "the information set has no possibilities for {card}"
                )
            }
            Self::NoConventionAction => {
                formatter.write_str("the convention admits no action in this position")
            }
        }
    }
}

impl std::error::Error for PolicyError {}
