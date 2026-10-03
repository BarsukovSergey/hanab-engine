//! Deterministic rules and player-safe observations for standard Hanabi.
//!
//! [`FullState`] is authoritative simulator truth. Code that chooses an action
//! should consume a [`PlayerView`] so hidden card identities cannot leak into a
//! policy or planning algorithm.

pub mod action;
pub mod card;
pub mod ids;
pub mod public_transition;
pub mod state;
pub mod view;
pub mod variant;

pub use action::{Action, Clue};
pub use card::{Card, Rank, Suit, standard_deck};
pub use ids::{CardId, PlayerId};
pub use state::{
    EndReason, FullState, GameEvent, GameStatus, HistoryEntry, InvariantViolation, MAX_CLUE_TOKENS,
    MAX_PLAYERS, MAX_STRIKES, MIN_PLAYERS, RuleError, STANDARD_DECK_SIZE, SetupError, TurnResult,
    WorldConstructionError, WorldTemplate, standard_hand_size,
};
pub use variant::{
    CardRank, ClueColorId, ClueRank, GenericClue, Identity, IdentityId, IdentitySpec,
    IdentityUniverse, MAX_IDENTITIES, SuitId, VariantSpec, VariantSpecError,
};
pub use view::{ClueFacts, ObservedCard, ObservedEvent, ObservedHistoryEntry, PlayerView};
