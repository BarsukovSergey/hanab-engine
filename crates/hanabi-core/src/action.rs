use core::fmt;

use crate::{Card, CardId, ClueColorId, ClueRank, GenericClue, PlayerId, Rank, Suit};

/// A standard color or rank clue.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Clue {
    Suit(Suit),
    Rank(Rank),
}

impl Clue {
    #[must_use]
    pub fn matches(self, card: Card) -> bool {
        match self {
            Self::Suit(suit) => card.suit == suit,
            Self::Rank(rank) => card.rank == rank,
        }
    }
}

/// An action selected by the current player.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Action {
    Play(CardId),
    Discard(CardId),
    Clue { target: PlayerId, clue: Clue },
}

/// A physical action whose clue meaning is supplied by the active variant.
/// No convention interpretation is attached to this action.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PhysicalAction {
    Play(CardId),
    Discard(CardId),
    Clue { target: PlayerId, clue: GenericClue },
}

/// Converts standard clue labels losslessly. The physical meaning is equivalent
/// for No Variant; other variants interpret these labels through their spec.
impl From<Clue> for GenericClue {
    fn from(clue: Clue) -> Self {
        match clue {
            Clue::Suit(suit) => Self::Color(ClueColorId::new(suit.value())),
            Clue::Rank(rank) => Self::Rank(ClueRank::new(rank.value())),
        }
    }
}

/// A physical clue has no equivalent in the legacy No Variant clue domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardClueError(pub GenericClue);

impl fmt::Display for StandardClueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "clue {:?} is outside the No Variant domain",
            self.0
        )
    }
}

impl std::error::Error for StandardClueError {}

impl TryFrom<GenericClue> for Clue {
    type Error = StandardClueError;

    fn try_from(clue: GenericClue) -> Result<Self, Self::Error> {
        match clue {
            GenericClue::Color(color) if color.value() < 5 => {
                Ok(Self::Suit(Suit::new(color.value())))
            }
            GenericClue::Rank(rank) if (1..=5).contains(&rank.value()) => {
                Ok(Self::Rank(Rank::new(rank.value())))
            }
            _ => Err(StandardClueError(clue)),
        }
    }
}

impl From<Action> for PhysicalAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Play(card) => Self::Play(card),
            Action::Discard(card) => Self::Discard(card),
            Action::Clue { target, clue } => Self::Clue {
                target,
                clue: clue.into(),
            },
        }
    }
}

impl TryFrom<PhysicalAction> for Action {
    type Error = StandardClueError;

    fn try_from(action: PhysicalAction) -> Result<Self, Self::Error> {
        match action {
            PhysicalAction::Play(card) => Ok(Self::Play(card)),
            PhysicalAction::Discard(card) => Ok(Self::Discard(card)),
            PhysicalAction::Clue { target, clue } => Ok(Self::Clue {
                target,
                clue: clue.try_into()?,
            }),
        }
    }
}
