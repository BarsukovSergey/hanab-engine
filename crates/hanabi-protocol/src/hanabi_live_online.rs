use core::fmt;
use std::collections::HashSet;

use hanabi_core::{
    Action, Card, CardId, CardRank, ClueColorId, ClueFacts, ClueRank, EndReason, GameStatus,
    GenericClue, MAX_CLUE_TOKENS, ObservedCard, ObservedEvent, ObservedHistoryEntry,
    PhysicalAction, PlayerId, PlayerView, StandardClueError, SuitId, VariantId, VariantSpec,
    standard_hand_size,
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::hanabi_live::{HanabiLiveActionType, variant_id_from_live_name};

/// A player-safe snapshot assembled from Hanabi Live init and scrubbed action
/// messages.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HanabiLiveSnapshot {
    #[serde(rename = "tableID")]
    table_id: u64,
    player_names: Vec<String>,
    our_player_index: usize,
    #[serde(default)]
    spectating: bool,
    #[serde(default)]
    replay: bool,
    options: HanabiLiveOnlineOptions,
    actions: Vec<HanabiLiveOnlineAction>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HanabiLiveOnlineOptions {
    variant_name: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct HanabiLiveOnlineClue {
    #[serde(rename = "type")]
    clue_type: HanabiLiveClueType,
    value: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HanabiLiveClueType {
    Suit,
    Rank,
    Unknown(u8),
}

impl HanabiLiveClueType {
    const fn from_code(code: u8) -> Self {
        match code {
            0 => Self::Suit,
            1 => Self::Rank,
            other => Self::Unknown(other),
        }
    }
}

impl<'de> Deserialize<'de> for HanabiLiveClueType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        u8::deserialize(deserializer).map(Self::from_code)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
enum HanabiLiveOnlineAction {
    #[serde(rename = "draw", rename_all = "camelCase")]
    Draw {
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
    },
    #[serde(rename = "clue", rename_all = "camelCase")]
    Clue {
        clue: HanabiLiveOnlineClue,
        giver: usize,
        list: Vec<usize>,
        target: usize,
        #[serde(default)]
        turn: Option<u32>,
    },
    #[serde(rename = "play", rename_all = "camelCase")]
    Play {
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
    },
    #[serde(rename = "discard", rename_all = "camelCase")]
    Discard {
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
        failed: bool,
    },
    #[serde(rename = "strike")]
    Strike { num: u8 },
    #[serde(rename = "status")]
    Status { clues: u8 },
    #[serde(rename = "turn", rename_all = "camelCase")]
    Turn { num: u32, current_player_index: i16 },
    #[serde(rename = "gameOver")]
    GameOver,
    #[serde(other)]
    Ignored,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum HanabiLiveSessionRequest {
    Initialize {
        snapshot: HanabiLiveSnapshot,
    },
    Append {
        #[serde(rename = "tableID")]
        table_id: u64,
        actions: Vec<HanabiLiveOnlineAction>,
    },
}

/// Incremental state for one persistent Hanabi Live engine process.
///
/// The first request initializes the session from the server's complete,
/// scrubbed action list. Later requests append only newly received actions,
/// avoiding replaying the full game history before every planning request.
#[derive(Clone, Debug, Default)]
pub struct HanabiLiveSessionState {
    session: Option<HanabiLiveSession>,
}

#[derive(Clone, Debug)]
struct HanabiLiveSession {
    table_id: u64,
    builder: LiveViewBuilder,
}

/// Wire payload accepted by the Hanabi Live action WebSocket command.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HanabiLiveActionCommand {
    #[serde(rename = "tableID")]
    pub table_id: u64,
    #[serde(rename = "type")]
    pub action_type: HanabiLiveActionType,
    pub target: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<u8>,
}

impl HanabiLiveSnapshot {
    /// Parses one live snapshot JSON document.
    ///
    /// # Errors
    ///
    /// Returns `LiveSnapshotError::Json` when the document does not match the
    /// live snapshot schema.
    pub fn from_json(json: &str) -> Result<Self, LiveSnapshotError> {
        serde_json::from_str(json).map_err(LiveSnapshotError::Json)
    }

    #[must_use]
    pub const fn table_id(&self) -> u64 {
        self.table_id
    }

    /// Reconstructs the exact observation legally available to the bot.
    ///
    /// The server scrubs the bot's own draws to -1. This adapter never accepts
    /// another player's hidden identity or fills an own identity from
    /// simulator truth.
    ///
    /// # Errors
    ///
    /// Returns `LiveSnapshotError` when the game is not an active-player game,
    /// the stream is malformed, or convention/policy support is unavailable.
    pub fn player_view(&self) -> Result<PlayerView, LiveSnapshotError> {
        HanabiLiveSession::from_snapshot(self)?.player_view()
    }

    /// Reconstructs a player-safe physical view for any supported catalog variant.
    /// Own-hand identities remain hidden, including after public clues.
    ///
    /// # Errors
    ///
    /// Returns [`LiveSnapshotError`] for unsupported variants, non-player tables,
    /// or malformed or incomplete scrubbed action streams.
    pub fn physical_player_view(&self) -> Result<PlayerView<GenericClue>, LiveSnapshotError> {
        HanabiLiveSession::from_snapshot(self)?.physical_player_view()
    }
}

impl HanabiLiveSessionState {
    /// Creates an empty session awaiting an `initialize` request.
    #[must_use]
    pub const fn new() -> Self {
        Self { session: None }
    }

    /// Applies one newline-delimited JSON session request and returns the
    /// current table and player view.
    ///
    /// Updates are transactional: malformed action batches leave the previous
    /// session state untouched so callers can restart or resynchronize safely.
    ///
    /// # Errors
    ///
    /// Returns [`LiveSnapshotError`] when the request is malformed, arrives
    /// before initialization, targets another table, contains invalid actions,
    /// or requires convention/policy support for a non-No-Variant game.
    pub fn apply_json(&mut self, json: &str) -> Result<(u64, PlayerView), LiveSnapshotError> {
        self.apply_json_with_view(json, HanabiLiveSession::player_view)
    }

    /// Applies an initialize/append request using physical rules only.
    /// The variant is fixed by initialization; append requests cannot supply metadata.
    /// Failed initialization or append leaves the previous valid session untouched.
    ///
    /// # Errors
    ///
    /// Returns [`LiveSnapshotError`] for malformed requests, unsupported variants,
    /// invalid actions, missing initialization, or a table mismatch.
    pub fn apply_physical_json(
        &mut self,
        json: &str,
    ) -> Result<(u64, PlayerView<GenericClue>), LiveSnapshotError> {
        self.apply_json_with_view(json, HanabiLiveSession::physical_player_view)
    }

    fn apply_json_with_view<V>(
        &mut self,
        json: &str,
        build_view: impl FnOnce(&HanabiLiveSession) -> Result<V, LiveSnapshotError>,
    ) -> Result<(u64, V), LiveSnapshotError> {
        let request: HanabiLiveSessionRequest =
            serde_json::from_str(json).map_err(LiveSnapshotError::Json)?;
        let candidate = match request {
            HanabiLiveSessionRequest::Initialize { snapshot } => {
                HanabiLiveSession::from_snapshot(&snapshot)?
            }
            HanabiLiveSessionRequest::Append { table_id, actions } => {
                let current = self
                    .session
                    .as_ref()
                    .ok_or(LiveSnapshotError::SessionNotInitialized)?;
                if current.table_id != table_id {
                    return Err(LiveSnapshotError::SessionTableMismatch {
                        expected: current.table_id,
                        actual: table_id,
                    });
                }
                let mut candidate = current.clone();
                candidate.append(&actions)?;
                candidate
            }
        };
        let view = build_view(&candidate)?;
        let table_id = candidate.table_id;
        self.session = Some(candidate);
        Ok((table_id, view))
    }
}

impl HanabiLiveSession {
    fn from_snapshot(snapshot: &HanabiLiveSnapshot) -> Result<Self, LiveSnapshotError> {
        if snapshot.spectating {
            return Err(LiveSnapshotError::Spectating);
        }
        if snapshot.replay {
            return Err(LiveSnapshotError::Replay);
        }
        let variant_id =
            variant_id_from_live_name(&snapshot.options.variant_name).ok_or_else(|| {
                LiveSnapshotError::UnsupportedVariant(snapshot.options.variant_name.clone())
            })?;
        if standard_hand_size(snapshot.player_names.len()).is_none() {
            return Err(LiveSnapshotError::InvalidPlayerCount(
                snapshot.player_names.len(),
            ));
        }
        if snapshot.our_player_index >= snapshot.player_names.len() {
            return Err(LiveSnapshotError::InvalidObserver(
                snapshot.our_player_index,
            ));
        }

        let mut builder = LiveViewBuilder::new(
            snapshot.player_names.len(),
            snapshot.our_player_index,
            VariantSpec::from_catalog(variant_id),
        );
        for action in &snapshot.actions {
            builder.apply(action)?;
        }
        Ok(Self {
            table_id: snapshot.table_id,
            builder,
        })
    }

    fn append(&mut self, actions: &[HanabiLiveOnlineAction]) -> Result<(), LiveSnapshotError> {
        for action in actions {
            self.builder.apply(action)?;
        }
        Ok(())
    }

    fn player_view(&self) -> Result<PlayerView, LiveSnapshotError> {
        if self.builder.variant != VariantSpec::from_catalog(VariantId::NoVariant) {
            return Err(LiveSnapshotError::PolicyUnsupportedVariant(
                self.builder.variant.name().to_owned(),
            ));
        }
        self.physical_player_view()?
            .try_standard()
            .map_err(LiveSnapshotError::StandardClue)
    }

    fn physical_player_view(&self) -> Result<PlayerView<GenericClue>, LiveSnapshotError> {
        self.builder.player_view()
    }
}

impl HanabiLiveActionCommand {
    /// Converts an engine action into the payload expected by Hanabi Live.
    #[must_use]
    pub fn from_engine_action(table_id: u64, action: Action) -> Self {
        Self::from_physical_action(table_id, action.into())
    }

    /// Encodes a physical action without convention interpretation.
    /// The caller supplies an action legal under the active variant.
    #[must_use]
    pub fn from_physical_action(table_id: u64, action: PhysicalAction) -> Self {
        match action {
            PhysicalAction::Play(card) => Self {
                table_id,
                action_type: HanabiLiveActionType::Play,
                target: card.index(),
                value: None,
            },
            PhysicalAction::Discard(card) => Self {
                table_id,
                action_type: HanabiLiveActionType::Discard,
                target: card.index(),
                value: None,
            },
            PhysicalAction::Clue { target, clue } => match clue {
                GenericClue::Color(color) => Self {
                    table_id,
                    action_type: HanabiLiveActionType::SuitClue,
                    target: target.index(),
                    value: Some(color.value()),
                },
                GenericClue::Rank(rank) => Self {
                    table_id,
                    action_type: HanabiLiveActionType::RankClue,
                    target: target.index(),
                    value: Some(rank.value()),
                },
            },
        }
    }
}

#[derive(Clone, Debug)]
struct LiveViewBuilder {
    variant: VariantSpec,
    observer: PlayerId,
    current_player: PlayerId,
    turn: u32,
    hands: Vec<Vec<ObservedCard>>,
    deck_size: usize,
    play_stacks: Vec<Vec<(CardId, Card)>>,
    discard_pile: Vec<(CardId, Card)>,
    clue_tokens: u8,
    strikes: u8,
    final_turns_remaining: Option<u8>,
    final_round_start_turn: Option<u32>,
    status: GameStatus,
    history: Vec<ObservedHistoryEntry<GenericClue>>,
    seen_cards: Vec<bool>,
    initial_deal_remaining: usize,
}

impl LiveViewBuilder {
    fn new(player_count: usize, observer: usize, variant: VariantSpec) -> Self {
        let hand_size = standard_hand_size(player_count).expect("validated standard player count");
        let total_cards = variant.identities().deck_size();
        let play_stacks =
            vec![Vec::with_capacity(variant.card_ranks().len()); variant.stack_suits().len()];
        Self {
            variant,
            observer: player_id(observer),
            current_player: player_id(0),
            turn: 0,
            hands: vec![Vec::with_capacity(hand_size); player_count],
            deck_size: total_cards,
            play_stacks,
            discard_pile: Vec::new(),
            clue_tokens: MAX_CLUE_TOKENS,
            strikes: 0,
            final_turns_remaining: None,
            final_round_start_turn: None,
            status: GameStatus::InProgress,
            history: Vec::new(),
            seen_cards: vec![false; total_cards],
            initial_deal_remaining: hand_size * player_count,
        }
    }

    fn apply(&mut self, action: &HanabiLiveOnlineAction) -> Result<(), LiveSnapshotError> {
        match action {
            HanabiLiveOnlineAction::Draw {
                player_index,
                order,
                suit_index,
                rank,
            } => self.draw(*player_index, *order, *suit_index, *rank),
            HanabiLiveOnlineAction::Clue {
                clue,
                giver,
                list,
                target,
                turn,
            } => self.clue(*clue, *giver, list, *target, *turn),
            HanabiLiveOnlineAction::Play {
                player_index,
                order,
                suit_index,
                rank,
            } => self.play(*player_index, *order, *suit_index, *rank),
            HanabiLiveOnlineAction::Discard {
                player_index,
                order,
                suit_index,
                rank,
                failed,
            } => self.discard(*player_index, *order, *suit_index, *rank, *failed),
            HanabiLiveOnlineAction::Strike { num } => {
                if *num == 0 || *num > 3 {
                    return Err(LiveSnapshotError::InvalidStrikeCount(*num));
                }
                self.strikes = *num;
                Ok(())
            }
            HanabiLiveOnlineAction::Status { clues } => {
                if *clues > MAX_CLUE_TOKENS {
                    return Err(LiveSnapshotError::InvalidClueTokenCount(*clues));
                }
                self.clue_tokens = *clues;
                Ok(())
            }
            HanabiLiveOnlineAction::Turn {
                num,
                current_player_index,
            } => self.advance_turn(*num, *current_player_index),
            HanabiLiveOnlineAction::GameOver => {
                self.status = if self.strikes >= 3 {
                    GameStatus::Finished(EndReason::TooManyStrikes)
                } else if self.play_stacks.iter().map(Vec::len).sum::<usize>()
                    == self.variant.max_score()
                {
                    GameStatus::Finished(EndReason::PerfectScore)
                } else {
                    GameStatus::Finished(EndReason::FinalRoundComplete)
                };
                Ok(())
            }
            HanabiLiveOnlineAction::Ignored => Ok(()),
        }
    }

    fn draw(
        &mut self,
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
    ) -> Result<(), LiveSnapshotError> {
        self.require_player(player_index)?;
        let card = self.new_card(order)?;
        let supplied_identity = parse_optional_identity(&self.variant, suit_index, rank)?;
        let identity = if player_index == self.observer.index() {
            None
        } else {
            Some(
                supplied_identity
                    .ok_or(LiveSnapshotError::HiddenTeammateCard { card, player_index })?,
            )
        };
        self.hands[player_index].push(ObservedCard {
            id: card,
            identity,
            clues: ClueFacts::default(),
        });
        self.deck_size = self
            .deck_size
            .checked_sub(1)
            .ok_or(LiveSnapshotError::TooManyDraws)?;

        if self.initial_deal_remaining > 0 {
            self.initial_deal_remaining -= 1;
        } else {
            self.history.push(ObservedHistoryEntry {
                turn: self.turn,
                event: ObservedEvent::Drew {
                    player: player_id(player_index),
                    card,
                    identity,
                },
            });
            if self.deck_size == 0 {
                let player_count = u8::try_from(self.hands.len())
                    .expect("standard Hanabi has at most six players");
                self.final_turns_remaining = Some(player_count);
                self.final_round_start_turn = Some(self.turn + 1);
            }
        }
        Ok(())
    }

    fn clue(
        &mut self,
        wire_clue: HanabiLiveOnlineClue,
        giver: usize,
        touched_orders: &[usize],
        target: usize,
        event_turn: Option<u32>,
    ) -> Result<(), LiveSnapshotError> {
        self.require_player(giver)?;
        self.require_player(target)?;
        let clue = parse_clue(&self.variant, wire_clue)?;
        let touched = touched_orders
            .iter()
            .copied()
            .map(|order| checked_card_id(&self.variant, order))
            .collect::<Result<HashSet<_>, _>>()?;
        if touched.len() != touched_orders.len() {
            return Err(LiveSnapshotError::DuplicateTouchedCard);
        }

        let mut touched_cards = Vec::new();
        let mut untouched_cards = Vec::new();
        for card in &mut self.hands[target] {
            if touched.contains(&card.id) {
                card.clues.record_physical(clue, true);
                touched_cards.push(card.id);
            } else {
                card.clues.record_physical(clue, false);
                untouched_cards.push(card.id);
            }
        }
        if touched_cards.len() != touched.len() {
            return Err(LiveSnapshotError::TouchedCardNotInHand);
        }
        self.clue_tokens = self
            .clue_tokens
            .checked_sub(1)
            .ok_or(LiveSnapshotError::ClueWithoutToken)?;
        self.history.push(ObservedHistoryEntry {
            turn: event_turn.unwrap_or(self.turn),
            event: ObservedEvent::Clued {
                giver: player_id(giver),
                target: player_id(target),
                clue,
                touched: touched_cards,
                untouched: untouched_cards,
            },
        });
        Ok(())
    }

    fn play(
        &mut self,
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
    ) -> Result<(), LiveSnapshotError> {
        self.require_player(player_index)?;
        let card = checked_card_id(&self.variant, order)?;
        let identity = parse_required_identity(&self.variant, card, suit_index, rank)?;
        self.remove_card(player_index, card, identity)?;
        let suit_index = self
            .variant
            .stack_suits()
            .iter()
            .position(|suit| *suit == identity.suit)
            .expect("parsed active physical suit");
        let stack = &mut self.play_stacks[suit_index];
        if self.variant.card_ranks().get(stack.len()) != Some(&identity.rank) {
            return Err(LiveSnapshotError::InvalidSuccessfulPlay { card, identity });
        }
        stack.push((card, identity));
        let identity_id = self
            .variant
            .identities()
            .id_for(identity.suit, identity.rank)
            .expect("parsed active identity");
        if self.variant.successor(identity_id).is_none() {
            self.clue_tokens = self.clue_tokens.saturating_add(1).min(MAX_CLUE_TOKENS);
        }
        self.history.push(ObservedHistoryEntry {
            turn: self.turn,
            event: ObservedEvent::Played {
                player: player_id(player_index),
                card,
                identity,
                successful: true,
            },
        });
        Ok(())
    }

    fn discard(
        &mut self,
        player_index: usize,
        order: usize,
        suit_index: i16,
        rank: i16,
        failed: bool,
    ) -> Result<(), LiveSnapshotError> {
        self.require_player(player_index)?;
        let card = checked_card_id(&self.variant, order)?;
        let identity = parse_required_identity(&self.variant, card, suit_index, rank)?;
        self.remove_card(player_index, card, identity)?;
        self.discard_pile.push((card, identity));
        let event = if failed {
            ObservedEvent::Played {
                player: player_id(player_index),
                card,
                identity,
                successful: false,
            }
        } else {
            self.clue_tokens = self.clue_tokens.saturating_add(1).min(MAX_CLUE_TOKENS);
            ObservedEvent::Discarded {
                player: player_id(player_index),
                card,
                identity,
            }
        };
        self.history.push(ObservedHistoryEntry {
            turn: self.turn,
            event,
        });
        if failed && self.strikes >= 3 {
            self.status = GameStatus::Finished(EndReason::TooManyStrikes);
        }
        Ok(())
    }

    fn advance_turn(
        &mut self,
        turn: u32,
        current_player_index: i16,
    ) -> Result<(), LiveSnapshotError> {
        if turn < self.turn {
            return Err(LiveSnapshotError::TurnWentBackward {
                previous: self.turn,
                next: turn,
            });
        }
        self.turn = turn;
        if let Some(start) = self.final_round_start_turn {
            let elapsed = turn.saturating_sub(start);
            let player_count =
                u8::try_from(self.hands.len()).expect("standard Hanabi has at most six players");
            self.final_turns_remaining =
                Some(player_count.saturating_sub(u8::try_from(elapsed).unwrap_or(u8::MAX)));
        }
        if current_player_index >= 0 {
            let current = usize::try_from(current_player_index)
                .expect("a nonnegative player index fits in usize");
            self.require_player(current)?;
            self.current_player = player_id(current);
        }
        Ok(())
    }

    fn remove_card(
        &mut self,
        player_index: usize,
        card: CardId,
        revealed: Card,
    ) -> Result<(), LiveSnapshotError> {
        let hand = &mut self.hands[player_index];
        let position = hand
            .iter()
            .position(|candidate| candidate.id == card)
            .ok_or(LiveSnapshotError::CardNotInHand { card, player_index })?;
        if let Some(visible) = hand[position].identity {
            if visible != revealed {
                return Err(LiveSnapshotError::IdentityChanged {
                    card,
                    visible,
                    revealed,
                });
            }
        }
        hand.remove(position);
        Ok(())
    }

    fn new_card(&mut self, order: usize) -> Result<CardId, LiveSnapshotError> {
        let card = checked_card_id(&self.variant, order)?;
        if self.seen_cards[order] {
            return Err(LiveSnapshotError::DuplicateCard(card));
        }
        self.seen_cards[order] = true;
        Ok(card)
    }

    fn require_player(&self, player_index: usize) -> Result<(), LiveSnapshotError> {
        if player_index >= self.hands.len() {
            return Err(LiveSnapshotError::InvalidPlayerIndex(player_index));
        }
        Ok(())
    }

    fn player_view(&self) -> Result<PlayerView<GenericClue>, LiveSnapshotError> {
        if self.initial_deal_remaining != 0 {
            return Err(LiveSnapshotError::IncompleteInitialDeal(
                self.initial_deal_remaining,
            ));
        }
        Ok(PlayerView {
            variant: self.variant.clone(),
            observer: self.observer,
            current_player: self.current_player,
            turn: self.turn,
            hands: self.hands.clone(),
            deck_size: self.deck_size,
            play_stacks: self.play_stacks.clone(),
            discard_pile: self.discard_pile.clone(),
            clue_tokens: self.clue_tokens,
            strikes: self.strikes,
            final_turns_remaining: self.final_turns_remaining,
            status: self.status,
            history: self.history.clone(),
        })
    }
}

fn player_id(index: usize) -> PlayerId {
    PlayerId::new(u8::try_from(index).expect("standard Hanabi has at most six players"))
}

fn checked_card_id(variant: &VariantSpec, order: usize) -> Result<CardId, LiveSnapshotError> {
    if order >= variant.identities().deck_size() {
        return Err(LiveSnapshotError::InvalidCardOrder(order));
    }
    Ok(CardId::new(order))
}

fn parse_optional_identity(
    variant: &VariantSpec,
    suit_index: i16,
    rank: i16,
) -> Result<Option<Card>, LiveSnapshotError> {
    match (suit_index, rank) {
        (-1, -1) => Ok(None),
        (-1, _) | (_, -1) => Err(LiveSnapshotError::PartiallyHiddenIdentity { suit_index, rank }),
        _ => Ok(Some(Card::new(
            parse_suit(variant, suit_index)?,
            parse_rank(variant, rank)?,
        ))),
    }
}

fn parse_required_identity(
    variant: &VariantSpec,
    card: CardId,
    suit_index: i16,
    rank: i16,
) -> Result<Card, LiveSnapshotError> {
    parse_optional_identity(variant, suit_index, rank)?
        .ok_or(LiveSnapshotError::HiddenPublicCard(card))
}

fn parse_suit(variant: &VariantSpec, value: i16) -> Result<SuitId, LiveSnapshotError> {
    usize::try_from(value)
        .ok()
        .and_then(|index| variant.stack_suits().get(index).copied())
        .ok_or(LiveSnapshotError::InvalidSuit(value))
}

fn parse_rank(variant: &VariantSpec, value: i16) -> Result<CardRank, LiveSnapshotError> {
    variant
        .card_ranks()
        .iter()
        .copied()
        .find(|rank| i16::from(rank.value()) == value)
        .ok_or(LiveSnapshotError::InvalidRank(value))
}

fn parse_clue(
    variant: &VariantSpec,
    clue: HanabiLiveOnlineClue,
) -> Result<GenericClue, LiveSnapshotError> {
    match clue.clue_type {
        HanabiLiveClueType::Suit => u8::try_from(clue.value)
            .ok()
            .map(ClueColorId::new)
            .filter(|color| variant.clue_colors().contains(color))
            .map(GenericClue::Color)
            .ok_or(LiveSnapshotError::InvalidClueColor(clue.value)),
        HanabiLiveClueType::Rank => u8::try_from(clue.value)
            .ok()
            .map(ClueRank::new)
            .filter(|rank| variant.clue_ranks().contains(rank))
            .map(GenericClue::Rank)
            .ok_or(LiveSnapshotError::InvalidClueRank(clue.value)),
        HanabiLiveClueType::Unknown(other) => Err(LiveSnapshotError::InvalidClueType(other)),
    }
}

/// Why a scrubbed Hanabi Live action stream could not become a player view.
#[derive(Debug)]
pub enum LiveSnapshotError {
    Json(serde_json::Error),
    Spectating,
    Replay,
    UnsupportedVariant(String),
    /// Physical rules are supported, but production conventions/policy are not.
    PolicyUnsupportedVariant(String),
    StandardClue(StandardClueError),
    InvalidPlayerCount(usize),
    InvalidObserver(usize),
    InvalidPlayerIndex(usize),
    InvalidCardOrder(usize),
    DuplicateCard(CardId),
    TooManyDraws,
    IncompleteInitialDeal(usize),
    HiddenTeammateCard {
        card: CardId,
        player_index: usize,
    },
    HiddenPublicCard(CardId),
    PartiallyHiddenIdentity {
        suit_index: i16,
        rank: i16,
    },
    InvalidSuit(i16),
    InvalidRank(i16),
    InvalidClueType(u8),
    InvalidClueColor(i16),
    InvalidClueRank(i16),
    DuplicateTouchedCard,
    TouchedCardNotInHand,
    ClueWithoutToken,
    InvalidClueTokenCount(u8),
    InvalidStrikeCount(u8),
    CardNotInHand {
        card: CardId,
        player_index: usize,
    },
    IdentityChanged {
        card: CardId,
        visible: Card,
        revealed: Card,
    },
    InvalidSuccessfulPlay {
        card: CardId,
        identity: Card,
    },
    TurnWentBackward {
        previous: u32,
        next: u32,
    },
    SessionNotInitialized,
    SessionTableMismatch {
        expected: u64,
        actual: u64,
    },
}

impl fmt::Display for LiveSnapshotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid live snapshot JSON: {error}"),
            Self::Spectating => formatter.write_str("the bot is spectating rather than playing"),
            Self::Replay => formatter.write_str("the table is a replay rather than a live game"),
            Self::UnsupportedVariant(variant) => {
                write!(formatter, "unsupported Hanabi Live variant {variant:?}")
            }
            Self::PolicyUnsupportedVariant(variant) => write!(
                formatter,
                "convention/policy support is not available yet for Hanabi Live variant {variant:?}; use the physical view API"
            ),
            Self::StandardClue(error) => error.fmt(formatter),
            Self::InvalidPlayerCount(count) => {
                write!(formatter, "live game has invalid player count {count}")
            }
            Self::InvalidObserver(index) => {
                write!(formatter, "bot player index {index} is out of range")
            }
            Self::InvalidPlayerIndex(index) => {
                write!(formatter, "action player index {index} is out of range")
            }
            Self::InvalidCardOrder(order) => {
                write!(
                    formatter,
                    "card order {order} is outside the active variant deck"
                )
            }
            Self::DuplicateCard(card) => write!(formatter, "card {card} was drawn more than once"),
            Self::TooManyDraws => formatter
                .write_str("action stream draws more cards than the active variant deck contains"),
            Self::IncompleteInitialDeal(remaining) => {
                write!(
                    formatter,
                    "initial deal is missing {remaining} draw actions"
                )
            }
            Self::HiddenTeammateCard { card, player_index } => write!(
                formatter,
                "teammate card {card} in player {player_index}'s hand was unexpectedly hidden"
            ),
            Self::HiddenPublicCard(card) => {
                write!(formatter, "public card {card} has a hidden identity")
            }
            Self::PartiallyHiddenIdentity { suit_index, rank } => write!(
                formatter,
                "card identity is only partly hidden (suit {suit_index}, rank {rank})"
            ),
            Self::InvalidSuit(suit) => write!(formatter, "invalid physical suit index {suit}"),
            Self::InvalidRank(rank) => write!(formatter, "invalid physical rank {rank}"),
            Self::InvalidClueColor(color) => write!(formatter, "invalid clue color label {color}"),
            Self::InvalidClueRank(rank) => write!(formatter, "invalid clue rank label {rank}"),
            Self::InvalidClueType(clue_type) => write!(formatter, "invalid clue type {clue_type}"),
            Self::DuplicateTouchedCard => {
                formatter.write_str("clue lists the same touched card more than once")
            }
            Self::TouchedCardNotInHand => {
                formatter.write_str("clue touches a card outside the target hand")
            }
            Self::ClueWithoutToken => {
                formatter.write_str("clue was given with no clue token available")
            }
            Self::InvalidClueTokenCount(count) => {
                write!(
                    formatter,
                    "server reported invalid clue-token count {count}"
                )
            }
            Self::InvalidStrikeCount(count) => {
                write!(formatter, "server reported invalid strike count {count}")
            }
            Self::CardNotInHand { card, player_index } => write!(
                formatter,
                "card {card} is not in player {player_index}'s hand"
            ),
            Self::IdentityChanged {
                card,
                visible,
                revealed,
            } => write!(
                formatter,
                "card {card} changed identity from {visible} to {revealed}"
            ),
            Self::InvalidSuccessfulPlay { card, identity } => {
                write!(
                    formatter,
                    "server reported {card} ({identity}) as a successful play"
                )
            }
            Self::TurnWentBackward { previous, next } => {
                write!(formatter, "turn went backward from {previous} to {next}")
            }
            Self::SessionNotInitialized => {
                formatter.write_str("live session received actions before initialization")
            }
            Self::SessionTableMismatch { expected, actual } => write!(
                formatter,
                "live session is for table {expected}, but update targets table {actual}"
            ),
        }
    }
}

impl std::error::Error for LiveSnapshotError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::StandardClue(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hanabi_core::{Clue, ObservedEvent, Rank, Suit, standard_deck};

    fn snapshot_json() -> String {
        serde_json::json!({
            "tableID": 17,
            "playerNames": ["Bot", "Alice"],
            "ourPlayerIndex": 0,
            "spectating": false,
            "replay": false,
            "options": {"variantName": "No Variant"},
            "actions": [
                {"type": "draw", "playerIndex": 0, "order": 0, "suitIndex": -1, "rank": -1},
                {"type": "draw", "playerIndex": 0, "order": 1, "suitIndex": -1, "rank": -1},
                {"type": "draw", "playerIndex": 0, "order": 2, "suitIndex": -1, "rank": -1},
                {"type": "draw", "playerIndex": 0, "order": 3, "suitIndex": -1, "rank": -1},
                {"type": "draw", "playerIndex": 0, "order": 4, "suitIndex": -1, "rank": -1},
                {"type": "draw", "playerIndex": 1, "order": 5, "suitIndex": 0, "rank": 1},
                {"type": "draw", "playerIndex": 1, "order": 6, "suitIndex": 0, "rank": 1},
                {"type": "draw", "playerIndex": 1, "order": 7, "suitIndex": 0, "rank": 1},
                {"type": "draw", "playerIndex": 1, "order": 8, "suitIndex": 0, "rank": 2},
                {"type": "draw", "playerIndex": 1, "order": 9, "suitIndex": 0, "rank": 2},
                {
                    "type": "clue",
                    "clue": {"type": 1, "value": 1},
                    "giver": 0,
                    "list": [5, 6, 7],
                    "target": 1,
                    "turn": 0
                },
                {"type": "status", "clues": 7, "score": 0, "maxScore": 25},
                {"type": "turn", "num": 1, "currentPlayerIndex": 1},
                {"type": "play", "playerIndex": 1, "order": 5, "suitIndex": 0, "rank": 1},
                {"type": "draw", "playerIndex": 1, "order": 10, "suitIndex": 2, "rank": 1},
                {"type": "status", "clues": 7, "score": 1, "maxScore": 25},
                {"type": "turn", "num": 2, "currentPlayerIndex": 0}
            ]
        })
        .to_string()
    }

    #[test]
    fn unknown_clue_type_retains_its_domain_error() {
        let clue = HanabiLiveOnlineClue {
            clue_type: HanabiLiveClueType::Unknown(7),
            value: 0,
        };
        assert!(matches!(
            parse_clue(&VariantSpec::no_variant(), clue),
            Err(LiveSnapshotError::InvalidClueType(7))
        ));
    }

    #[test]
    fn reconstructs_a_player_safe_live_view() {
        let snapshot = HanabiLiveSnapshot::from_json(&snapshot_json()).unwrap();
        let view = snapshot.player_view().unwrap();

        assert_eq!(snapshot.table_id(), 17);
        assert_eq!(view.observer, PlayerId::new(0));
        assert_eq!(view.current_player, PlayerId::new(0));
        assert_eq!(view.turn, 2);
        assert_eq!(view.deck_size, 39);
        assert_eq!(view.clue_tokens, 7);
        assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
        assert_eq!(view.hands[1].last().unwrap().id, CardId::new(10));
        assert_eq!(view.play_stacks[Suit::Red.index()][0].0, CardId::new(5));
        assert_eq!(view.history.len(), 3);
        assert!(matches!(
            &view.history[0].event,
            ObservedEvent::Clued {
                clue: Clue::Rank(Rank::One),
                touched,
                ..
            } if touched == &[CardId::new(5), CardId::new(6), CardId::new(7)]
        ));
        assert!(matches!(
            view.history[1].event,
            ObservedEvent::Played {
                card,
                successful: true,
                ..
            } if card == CardId::new(5)
        ));
        assert!(matches!(
            view.history[2].event,
            ObservedEvent::Drew {
                card,
                identity: Some(Card {
                    suit: Suit::Green,
                    rank: Rank::One
                }),
                ..
            } if card == CardId::new(10)
        ));
    }

    #[test]
    fn reconstructs_six_player_initial_deal_with_three_card_hands() {
        let deck = standard_deck();
        let mut actions = Vec::new();
        for player_index in 0..6 {
            for slot in 0..3 {
                let order = player_index * 3 + slot;
                let identity = deck[order];
                let (suit_index, rank) = if player_index == 0 {
                    (-1, -1)
                } else {
                    (
                        i16::try_from(identity.suit.index()).unwrap(),
                        i16::from(identity.rank.number()),
                    )
                };
                actions.push(serde_json::json!({
                    "type": "draw",
                    "playerIndex": player_index,
                    "order": order,
                    "suitIndex": suit_index,
                    "rank": rank,
                }));
            }
        }
        let json = serde_json::json!({
            "tableID": 18,
            "playerNames": ["Bot", "Alice", "Bob", "Cathy", "Donald", "Emily"],
            "ourPlayerIndex": 0,
            "spectating": false,
            "replay": false,
            "options": {"variantName": "No Variant"},
            "actions": actions,
        });
        let snapshot = HanabiLiveSnapshot::from_json(&json.to_string()).unwrap();
        let view = snapshot.player_view().unwrap();

        assert_eq!(view.hands.len(), 6);
        assert!(view.hands.iter().all(|hand| hand.len() == 3));
        assert_eq!(view.deck_size, 32);
        assert!(view.hands[0].iter().all(|card| card.identity.is_none()));
    }

    #[test]
    fn live_session_appends_actions_incrementally_and_transactionally() {
        let mut snapshot: serde_json::Value = serde_json::from_str(&snapshot_json()).unwrap();
        let actions = snapshot["actions"].as_array_mut().unwrap();
        let appended = actions.split_off(13);
        let initialize = serde_json::json!({
            "kind": "initialize",
            "snapshot": snapshot,
        });
        let mut session = HanabiLiveSessionState::new();

        let (table_id, first) = session.apply_json(&initialize.to_string()).unwrap();
        assert_eq!(table_id, 17);
        assert_eq!(first.turn, 1);
        assert_eq!(first.current_player, PlayerId::new(1));

        let wrong_table = serde_json::json!({
            "kind": "append",
            "tableID": 99,
            "actions": appended,
        });
        assert!(matches!(
            session.apply_json(&wrong_table.to_string()),
            Err(LiveSnapshotError::SessionTableMismatch {
                expected: 17,
                actual: 99
            })
        ));

        let append = serde_json::json!({
            "kind": "append",
            "tableID": 17,
            "actions": wrong_table["actions"],
        });
        let (_, second) = session.apply_json(&append.to_string()).unwrap();
        assert_eq!(second.turn, 2);
        assert_eq!(second.current_player, PlayerId::new(0));
        assert_eq!(second.play_stacks[Suit::Red.index()].len(), 1);
        assert_eq!(second.hands[1].last().unwrap().id, CardId::new(10));
    }

    #[test]
    fn live_session_requires_initialization() {
        let mut session = HanabiLiveSessionState::new();
        let append = serde_json::json!({
            "kind": "append",
            "tableID": 17,
            "actions": [],
        });
        assert!(matches!(
            session.apply_json(&append.to_string()),
            Err(LiveSnapshotError::SessionNotInitialized)
        ));
    }

    #[test]
    fn engine_actions_map_to_server_wire_values() {
        assert_eq!(
            HanabiLiveActionCommand::from_engine_action(
                9,
                Action::Clue {
                    target: PlayerId::new(1),
                    clue: Clue::Suit(Suit::Purple),
                },
            ),
            HanabiLiveActionCommand {
                table_id: 9,
                action_type: HanabiLiveActionType::SuitClue,
                target: 1,
                value: Some(4),
            }
        );
        assert_eq!(
            HanabiLiveActionCommand::from_engine_action(9, Action::Play(CardId::new(12))),
            HanabiLiveActionCommand {
                table_id: 9,
                action_type: HanabiLiveActionType::Play,
                target: 12,
                value: None,
            }
        );
    }
}
