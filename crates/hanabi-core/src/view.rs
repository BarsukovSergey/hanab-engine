use crate::{
    Action, Card, CardId, Clue, ClueColorId, ClueRank, FullState, GameEvent, GameStatus,
    GenericClue, MAX_CLUE_TOKENS, PlayerId, Rank, Suit, VariantSpec,
};

/// Objective positive and negative clue facts attached to one physical card.
/// These are derived from authoritative history rather than stored as a second
/// source of truth.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ClueFacts {
    positive_suits: u8,
    negative_suits: u8,
    positive_ranks: u8,
    negative_ranks: u8,
}

impl ClueFacts {
    /// Whether an identity is consistent with direct positive and negative clues.
    #[must_use]
    pub fn allows(&self, card: Card) -> bool {
        let suit = 1 << card.suit.index();
        let rank = 1 << card.rank.index();
        (self.positive_suits == 0 || self.positive_suits & suit != 0)
            && self.negative_suits & suit == 0
            && (self.positive_ranks == 0 || self.positive_ranks & rank != 0)
            && self.negative_ranks & rank == 0
    }

    /// Bit set of all 25 standard identities allowed by these direct clues.
    /// Identity bits use [`Card::index`] order.
    #[must_use]
    pub fn identity_mask(self) -> u32 {
        let allowed_suits = if self.positive_suits == 0 {
            0b1_1111
        } else {
            self.positive_suits
        } & !self.negative_suits;
        let allowed_ranks = if self.positive_ranks == 0 {
            0b1_1111
        } else {
            self.positive_ranks
        } & !self.negative_ranks;

        (0..5).fold(0, |mask, suit| {
            mask | ((u32::from(allowed_suits & (1 << suit) != 0) * u32::from(allowed_ranks))
                << (suit * 5))
        })
    }

    /// Direct clue mask in the active universe's [`crate::IdentityId`] order.
    /// Legacy clue values name clue colors/ranks, whose physical touches come
    /// from the variant rather than from a card's suit/rank equality.
    #[must_use]
    pub fn identity_mask_in(self, variant: &VariantSpec) -> u64 {
        if self.is_empty() {
            return variant
                .identities()
                .identities()
                .iter()
                .fold(0, |mask, identity| mask | (1_u64 << identity.id.value()));
        }
        variant
            .identities()
            .identities()
            .iter()
            .fold(0, |mask, identity| {
                let colors = (0_u8..5).fold(0_u8, |bits, value| {
                    bits | (u8::from(
                        (self.positive_suits | self.negative_suits) & (1 << value) != 0
                            && variant
                                .touches(identity.id, GenericClue::Color(ClueColorId::new(value))),
                    ) << value)
                });
                let ranks = (0_u8..5).fold(0_u8, |bits, value| {
                    bits | (u8::from(
                        (self.positive_ranks | self.negative_ranks) & (1 << value) != 0
                            && variant
                                .touches(identity.id, GenericClue::Rank(ClueRank::new(value + 1))),
                    ) << value)
                });
                let allows = (self.positive_suits == 0 || self.positive_suits & colors != 0)
                    && self.negative_suits & colors == 0
                    && (self.positive_ranks == 0 || self.positive_ranks & ranks != 0)
                    && self.negative_ranks & ranks == 0;
                mask | (u64::from(allows) << identity.id.value())
            })
    }

    /// Records a direct positive clue fact.
    pub fn add_positive_clue(&mut self, clue: Clue) {
        self.record(clue, true);
    }

    /// Records a direct negative clue fact.
    pub fn add_negative_clue(&mut self, clue: Clue) {
        self.record(clue, false);
    }

    /// Whether this card was positively touched by `clue`.
    #[must_use]
    pub fn has_positive_clue(self, clue: Clue) -> bool {
        match clue {
            Clue::Suit(value) => self.positive_suits & (1 << value.index()) != 0,
            Clue::Rank(value) => self.positive_ranks & (1 << value.index()) != 0,
        }
    }

    /// Whether this card was negatively excluded by `clue`.
    #[must_use]
    pub fn has_negative_clue(self, clue: Clue) -> bool {
        match clue {
            Clue::Suit(value) => self.negative_suits & (1 << value.index()) != 0,
            Clue::Rank(value) => self.negative_ranks & (1 << value.index()) != 0,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.positive_suits == 0
            && self.negative_suits == 0
            && self.positive_ranks == 0
            && self.negative_ranks == 0
    }

    pub(crate) fn record(&mut self, clue: Clue, positive: bool) {
        match (clue, positive) {
            (Clue::Suit(value), true) => self.positive_suits |= 1 << value.index(),
            (Clue::Suit(value), false) => self.negative_suits |= 1 << value.index(),
            (Clue::Rank(value), true) => self.positive_ranks |= 1 << value.index(),
            (Clue::Rank(value), false) => self.negative_ranks |= 1 << value.index(),
        }
    }
}

/// A card as legally observed by one player.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObservedCard {
    pub id: CardId,
    /// `None` only for a card in the observer's own hand.
    pub identity: Option<Card>,
    pub clues: ClueFacts,
}

/// A history event projected for one player.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ObservedEvent {
    Clued {
        giver: PlayerId,
        target: PlayerId,
        clue: Clue,
        touched: Vec<CardId>,
        untouched: Vec<CardId>,
    },
    Played {
        player: PlayerId,
        card: CardId,
        identity: Card,
        successful: bool,
    },
    Discarded {
        player: PlayerId,
        card: CardId,
        identity: Card,
    },
    Drew {
        player: PlayerId,
        card: CardId,
        /// Hidden when the observer drew the card; visible to every other player.
        identity: Option<Card>,
    },
}

/// A turn-numbered observed event.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObservedHistoryEntry {
    pub turn: u32,
    pub event: ObservedEvent,
}

/// Everything one player is legally permitted to observe.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlayerView {
    /// Public physical rules and identity ordering; contains no deal or hidden cards.
    pub variant: VariantSpec,
    pub observer: PlayerId,
    pub current_player: PlayerId,
    pub turn: u32,
    pub hands: Vec<Vec<ObservedCard>>,
    pub deck_size: usize,
    pub play_stacks: Vec<Vec<(CardId, Card)>>,
    pub discard_pile: Vec<(CardId, Card)>,
    pub clue_tokens: u8,
    pub strikes: u8,
    pub final_turns_remaining: Option<u8>,
    pub status: GameStatus,
    pub history: Vec<ObservedHistoryEntry>,
}

impl PlayerView {
    /// Legal actions in stable order. Returns no actions when this view does
    /// not belong to the current player or the game has ended.
    ///
    /// # Panics
    ///
    /// Panics only if a view constructed by the core contains an invalid
    /// player count or omits the observer's hand.
    #[must_use]
    pub fn legal_actions(&self) -> Vec<Action> {
        if self.observer != self.current_player || self.status != GameStatus::InProgress {
            return Vec::new();
        }

        let own_hand = &self.hands[self.observer.index()];
        let mut actions = Vec::with_capacity(own_hand.len() * 2 + 40);
        actions.extend(own_hand.iter().map(|card| Action::Play(card.id)));
        if self.clue_tokens < MAX_CLUE_TOKENS {
            actions.extend(own_hand.iter().map(|card| Action::Discard(card.id)));
        }

        if self.clue_tokens > 0 {
            for (target_index, target_hand) in self.hands.iter().enumerate() {
                if target_index == self.observer.index() {
                    continue;
                }
                let target = PlayerId::new(
                    target_index
                        .try_into()
                        .expect("standard Hanabi has at most six players"),
                );

                for suit in Suit::ALL {
                    if target_hand
                        .iter()
                        .any(|card| card.identity.is_some_and(|identity| identity.suit == suit))
                    {
                        actions.push(Action::Clue {
                            target,
                            clue: Clue::Suit(suit),
                        });
                    }
                }
                for rank in Rank::ALL {
                    if target_hand
                        .iter()
                        .any(|card| card.identity.is_some_and(|identity| identity.rank == rank))
                    {
                        actions.push(Action::Clue {
                            target,
                            clue: Clue::Rank(rank),
                        });
                    }
                }
            }
        }
        actions
    }
}

impl FullState {
    /// Projects authoritative state into the legal observation for `observer`.
    ///
    /// # Panics
    ///
    /// Panics only if the full state violates its internal card-location
    /// invariant.
    #[must_use]
    pub fn view_for(&self, observer: PlayerId) -> Option<PlayerView> {
        self.build_view(observer)
    }

    fn build_view(&self, observer: PlayerId) -> Option<PlayerView> {
        if observer.index() >= usize::from(self.num_players()) {
            return None;
        }

        let hands = self
            .hands()
            .iter()
            .enumerate()
            .map(|(player_index, hand)| {
                hand.iter()
                    .map(|id| ObservedCard {
                        id: *id,
                        identity: (player_index != observer.index())
                            .then(|| self.card(*id).expect("located cards have identities")),
                        clues: *self.clue_facts(*id),
                    })
                    .collect()
            })
            .collect();

        let play_stacks = self
            .play_stacks()
            .iter()
            .map(|stack| {
                stack
                    .iter()
                    .map(|id| (*id, self.card(*id).expect("located cards have identities")))
                    .collect()
            })
            .collect();
        let discard_pile = self
            .discard_pile()
            .iter()
            .map(|id| (*id, self.card(*id).expect("located cards have identities")))
            .collect();
        let history = self
            .history()
            .iter()
            .map(|entry| ObservedHistoryEntry {
                turn: entry.turn,
                event: observed_event(self, observer, &entry.event),
            })
            .collect();

        Some(PlayerView {
            variant: self.variant().clone(),
            observer,
            current_player: self.current_player(),
            turn: self.turn(),
            hands,
            deck_size: self.deck_size(),
            play_stacks,
            discard_pile,
            clue_tokens: self.clue_tokens(),
            strikes: self.strikes(),
            final_turns_remaining: self.final_turns_remaining(),
            status: self.status(),
            history,
        })
    }
}

fn observed_event(state: &FullState, observer: PlayerId, event: &GameEvent) -> ObservedEvent {
    match event {
        GameEvent::Clued {
            giver,
            target,
            clue,
            touched,
            untouched,
        } => ObservedEvent::Clued {
            giver: *giver,
            target: *target,
            clue: *clue,
            touched: touched.clone(),
            untouched: untouched.clone(),
        },
        GameEvent::Played {
            player,
            card,
            successful,
        } => ObservedEvent::Played {
            player: *player,
            card: *card,
            identity: state.card(*card).expect("history cards have identities"),
            successful: *successful,
        },
        GameEvent::Discarded { player, card } => ObservedEvent::Discarded {
            player: *player,
            card: *card,
            identity: state.card(*card).expect("history cards have identities"),
        },
        GameEvent::Drew { player, card } => ObservedEvent::Drew {
            player: *player,
            card: *card,
            identity: (*player != observer)
                .then(|| state.card(*card).expect("history cards have identities")),
        },
    }
}
