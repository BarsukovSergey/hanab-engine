use core::fmt;

use hanabi_core::{
    Card, CardId, ClueColorId, ClueRank, FullState, GameStatus, GenericClue, PhysicalAction,
    PlayerId, RuleError, SetupError, VariantId, VariantSpec,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The compact JSON replay exported by Hanabi Live's `/copy` command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HanabiLiveReplay {
    pub seed: Option<String>,
    pub players: Vec<String>,
    pub deck: Vec<HanabiLiveCard>,
    pub actions: Vec<HanabiLiveAction>,
    pub options: Option<HanabiLiveOptions>,
}

#[derive(Deserialize)]
struct WireReplay {
    players: Option<Vec<String>>,
    deck: Option<Vec<HanabiLiveCard>>,
    seed: Option<String>,
    actions: Vec<HanabiLiveAction>,
    options: Option<HanabiLiveOptions>,
}

impl<'de> Deserialize<'de> for HanabiLiveReplay {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_wire(WireReplay::deserialize(deserializer)?)
    }
}

impl HanabiLiveReplay {
    fn from_wire<E: serde::de::Error>(wire: WireReplay) -> Result<Self, E> {
        let players = if let Some(players) = wire.players {
            players
        } else {
            let seed = wire
                .seed
                .as_deref()
                .ok_or_else(|| E::custom("replay requires players when no seed is provided"))?;
            let count = crate::seed::player_count_from_seed(seed).map_err(E::custom)?;
            // Hanab Live's known default names cover the original 2–5
            // player exports. Six-player replays must preserve explicit names
            // until the server's sixth default-name source is pinned here.
            let defaults = ["Alice", "Bob", "Cathy", "Donald", "Emily"];
            let names = defaults
                .get(..count)
                .ok_or_else(|| E::custom("six-player seeded replays require explicit players"))?;
            names.iter().map(|name| (*name).to_owned()).collect()
        };
        // Explicit decks remain authoritative, including custom/edited deals
        // whose original seed is still present as descriptive metadata.
        let deck = if let Some(deck) = wire.deck {
            deck
        } else {
            if wire
                .options
                .as_ref()
                .is_some_and(|options| options.variant != "No Variant")
            {
                return Err(E::custom("seed generation only supports No Variant"));
            }
            let seed = wire
                .seed
                .as_deref()
                .ok_or_else(|| E::custom("replay requires a deck or seed"))?;
            crate::seed::deck_from_seed(seed, players.len()).map_err(E::custom)?
        };
        Ok(Self {
            seed: wire.seed,
            players,
            deck,
            actions: wire.actions,
            options: wire.options,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct HanabiLiveCard {
    #[serde(rename = "suitIndex")]
    pub suit_index: u8,
    pub rank: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct HanabiLiveAction {
    #[serde(rename = "type")]
    pub action_type: HanabiLiveActionType,
    pub target: usize,
    #[serde(default)]
    pub value: u8,
}

/// Numeric action discriminant used by Hanabi Live replay and action payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HanabiLiveActionType {
    Play,
    Discard,
    SuitClue,
    RankClue,
    GameOver,
    /// Retained until semantic validation so callers receive a precise
    /// unsupported-action error rather than a generic JSON error.
    Unknown(u8),
}

impl HanabiLiveActionType {
    #[must_use]
    pub const fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Play,
            1 => Self::Discard,
            2 => Self::SuitClue,
            3 => Self::RankClue,
            4 => Self::GameOver,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Play => 0,
            Self::Discard => 1,
            Self::SuitClue => 2,
            Self::RankClue => 3,
            Self::GameOver => 4,
            Self::Unknown(code) => code,
        }
    }
}

impl Serialize for HanabiLiveActionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.code())
    }
}

impl<'de> Deserialize<'de> for HanabiLiveActionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        u8::deserialize(deserializer).map(Self::from_code)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct HanabiLiveOptions {
    #[serde(default = "default_variant")]
    pub variant: String,
}

fn default_variant() -> String {
    "No Variant".to_owned()
}

impl HanabiLiveReplay {
    /// Parses a compact Hanabi Live replay.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError::Json`] if the JSON does not match the export
    /// schema, or [`ReplayError::UnsupportedVariant`] for an unknown name.
    pub fn from_json(json: &str) -> Result<Self, ReplayError> {
        let wire: WireReplay = serde_json::from_str(json).map_err(ReplayError::Json)?;
        variant_from_options(wire.options.as_ref())?;
        Self::from_wire(wire).map_err(ReplayError::Json)
    }

    /// Replays catalog variants through the physical rules in `hanabi-core`.
    ///
    /// Both current exports and legacy fixtures without an `options` object
    /// are accepted. A missing variant is interpreted as `No Variant`.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError`] for unsupported variants or values, malformed
    /// decks, illegal actions, or an external game-over action that occurs
    /// before the rules engine reaches a terminal state.
    pub fn replay(&self) -> Result<FullState, ReplayError> {
        let mut state = self.initial_state()?;

        for (turn, action) in self.actions.iter().enumerate() {
            if action.action_type == HanabiLiveActionType::GameOver {
                if state.status() == GameStatus::InProgress {
                    return Err(ReplayError::ExternalGameOver { turn });
                }
                break;
            }

            let engine_action = action_from_live(action, state.variant())?;
            state
                .apply_physical(engine_action)
                .map_err(|source| ReplayError::IllegalAction { turn, source })?;
        }
        Ok(state)
    }

    /// Replays exactly `turn` game actions and returns the position before the
    /// next action. Turn zero is the initial deal.
    ///
    /// Unlike [`Self::replay`], this intentionally does not validate actions
    /// after the requested prefix. This allows analysis of an otherwise
    /// incomplete or subsequently malformed replay.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayError`] for invalid setup or prefix actions, or
    /// [`ReplayError::TurnOutOfRange`] if the replay ends before `turn`.
    pub fn state_at_turn(&self, turn: u32) -> Result<FullState, ReplayError> {
        let mut state = self.initial_state()?;
        if turn == 0 {
            return Ok(state);
        }

        for (action_index, action) in self.actions.iter().enumerate() {
            if action.action_type == HanabiLiveActionType::GameOver {
                if state.status() == GameStatus::InProgress {
                    return Err(ReplayError::ExternalGameOver { turn: action_index });
                }
                break;
            }

            let engine_action = action_from_live(action, state.variant())?;
            state
                .apply_physical(engine_action)
                .map_err(|source| ReplayError::IllegalAction {
                    turn: action_index,
                    source,
                })?;
            if state.turn() == turn {
                return Ok(state);
            }
        }

        Err(ReplayError::TurnOutOfRange {
            requested: turn,
            available: state.turn(),
        })
    }

    fn initial_state(&self) -> Result<FullState, ReplayError> {
        let variant = VariantSpec::from_catalog(variant_from_options(self.options.as_ref())?);

        let player_count = u8::try_from(self.players.len())
            .map_err(|_| ReplayError::InvalidPlayerCount(self.players.len()))?;
        let deck = self
            .deck
            .iter()
            .copied()
            .map(|card| card_from_live(card, &variant))
            .collect::<Result<Vec<_>, _>>()?;
        FullState::new_variant(player_count, variant, deck).map_err(ReplayError::Setup)
    }
}

// Exact wire names are owned by the protocol, not by the core catalog keys.
fn variant_from_options(options: Option<&HanabiLiveOptions>) -> Result<VariantId, ReplayError> {
    let name = options.map_or("No Variant", |options| options.variant.as_str());
    match name {
        "No Variant" => Ok(VariantId::NoVariant),
        "Black (6 Suits)" => Ok(VariantId::Black6Suits),
        "Rainbow (6 Suits)" => Ok(VariantId::Rainbow6Suits),
        "White (6 Suits)" => Ok(VariantId::White6Suits),
        "Pink (6 Suits)" => Ok(VariantId::Pink6Suits),
        "Brown (6 Suits)" => Ok(VariantId::Brown6Suits),
        "Prism (6 Suits)" => Ok(VariantId::Prism6Suits),
        "Omni (6 Suits)" => Ok(VariantId::Omni6Suits),
        "Null (6 Suits)" => Ok(VariantId::Null6Suits),
        _ => Err(ReplayError::UnsupportedVariant(name.to_owned())),
    }
}

fn card_from_live(card: HanabiLiveCard, variant: &VariantSpec) -> Result<Card, ReplayError> {
    let suit = variant
        .stack_suits()
        .get(usize::from(card.suit_index))
        .copied()
        .ok_or(ReplayError::InvalidSuit(card.suit_index))?;
    let rank = variant
        .card_ranks()
        .iter()
        .copied()
        .find(|rank| rank.value() == card.rank)
        .ok_or(ReplayError::InvalidRank(card.rank))?;
    Ok(Card::new(suit, rank))
}

fn action_from_live(
    action: &HanabiLiveAction,
    variant: &VariantSpec,
) -> Result<PhysicalAction, ReplayError> {
    match action.action_type {
        HanabiLiveActionType::Play => Ok(PhysicalAction::Play(CardId::new(action.target))),
        HanabiLiveActionType::Discard => Ok(PhysicalAction::Discard(CardId::new(action.target))),
        HanabiLiveActionType::SuitClue => Ok(PhysicalAction::Clue {
            target: player_from_target(action.target)?,
            clue: validate_clue(GenericClue::Color(ClueColorId::new(action.value)), variant)?,
        }),
        HanabiLiveActionType::RankClue => Ok(PhysicalAction::Clue {
            target: player_from_target(action.target)?,
            clue: validate_clue(GenericClue::Rank(ClueRank::new(action.value)), variant)?,
        }),
        HanabiLiveActionType::GameOver => unreachable!("game-over actions are handled by replay"),
        HanabiLiveActionType::Unknown(other) => Err(ReplayError::UnknownActionType(other)),
    }
}

fn player_from_target(target: usize) -> Result<PlayerId, ReplayError> {
    u8::try_from(target)
        .map(PlayerId::new)
        .map_err(|_| ReplayError::InvalidPlayerTarget(target))
}

fn validate_clue(clue: GenericClue, variant: &VariantSpec) -> Result<GenericClue, ReplayError> {
    let valid = match clue {
        GenericClue::Color(color) => variant.clue_colors().contains(&color),
        GenericClue::Rank(rank) => variant.clue_ranks().contains(&rank),
    };
    if valid {
        Ok(clue)
    } else {
        Err(ReplayError::InvalidClue(clue))
    }
}

#[derive(Debug)]
pub enum ReplayError {
    Json(serde_json::Error),
    UnsupportedVariant(String),
    InvalidPlayerCount(usize),
    InvalidPlayerTarget(usize),
    InvalidSuit(u8),
    InvalidRank(u8),
    InvalidClue(GenericClue),
    UnknownActionType(u8),
    Setup(SetupError),
    IllegalAction { turn: usize, source: RuleError },
    ExternalGameOver { turn: usize },
    TurnOutOfRange { requested: u32, available: u32 },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid Hanabi Live JSON: {error}"),
            Self::UnsupportedVariant(variant) => {
                write!(formatter, "unsupported Hanabi Live variant: {variant}")
            }
            Self::InvalidPlayerCount(count) => write!(formatter, "invalid player count: {count}"),
            Self::InvalidPlayerTarget(target) => {
                write!(formatter, "invalid player target: {target}")
            }
            Self::InvalidSuit(suit) => write!(formatter, "invalid variant suit index: {suit}"),
            Self::InvalidRank(rank) => write!(formatter, "invalid variant card rank: {rank}"),
            Self::InvalidClue(clue) => write!(formatter, "invalid variant clue label: {clue:?}"),
            Self::UnknownActionType(action) => {
                write!(formatter, "unknown Hanabi Live action type: {action}")
            }
            Self::Setup(error) => write!(formatter, "could not set up replay: {error}"),
            Self::IllegalAction { turn, source } => {
                write!(formatter, "illegal replay action at turn {turn}: {source}")
            }
            Self::ExternalGameOver { turn } => write!(
                formatter,
                "external game-over action at turn {turn} is not a physical rules ending"
            ),
            Self::TurnOutOfRange {
                requested,
                available,
            } => write!(
                formatter,
                "requested turn {requested}, but the replay contains only {available} game turns"
            ),
        }
    }
}

impl std::error::Error for ReplayError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Setup(error) => Some(error),
            Self::IllegalAction { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_types_round_trip_as_numeric_wire_values() {
        assert_eq!(
            serde_json::to_string(&HanabiLiveActionType::SuitClue).unwrap(),
            "2"
        );
        assert_eq!(
            serde_json::from_str::<HanabiLiveActionType>("9").unwrap(),
            HanabiLiveActionType::Unknown(9)
        );
        let action = HanabiLiveAction {
            action_type: HanabiLiveActionType::Unknown(9),
            target: 0,
            value: 0,
        };
        assert!(matches!(
            action_from_live(&action, &VariantSpec::no_variant()),
            Err(ReplayError::UnknownActionType(9))
        ));
    }
}
