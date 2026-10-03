use crate::{
    Action, Card, CardId, Clue, ClueColorId, ClueRank, FullState, GameEvent, GameStatus,
    GenericClue, MAX_CLUE_TOKENS, PhysicalAction, PlayerId, Rank, StandardClueError, Suit,
    VariantSpec,
};

/// Objective positive and negative clue facts attached to one physical card.
/// These are derived from authoritative history rather than stored as a second
/// source of truth.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ClueFacts {
    positive_suits: [u64; 4],
    negative_suits: [u64; 4],
    positive_ranks: [u64; 4],
    negative_ranks: [u64; 4],
}

impl ClueFacts {
    /// Whether a No Variant identity satisfies direct standard clue facts.
    #[must_use]
    pub fn allows(&self, card: Card) -> bool {
        let suit = u64::from(card.suit.value());
        let rank = u64::from(card.rank.value());
        (self.positive_suits == [0; 4] || Self::contains(self.positive_suits, suit))
            && !Self::contains(self.negative_suits, suit)
            && (self.positive_ranks == [0; 4] || Self::contains(self.positive_ranks, rank))
            && !Self::contains(self.negative_ranks, rank)
    }

    /// Bit set of standard identities allowed by direct standard clue facts.
    /// Identity bits use [`Card::index`] order.
    #[must_use]
    pub fn identity_mask(self) -> u32 {
        let suits = if self.positive_suits == [0; 4] {
            0b1_1111
        } else {
            self.positive_suits[0].to_le_bytes()[0] & 0b1_1111
        } & !self.negative_suits[0].to_le_bytes()[0];
        let ranks = if self.positive_ranks == [0; 4] {
            0b1_1111
        } else {
            (self.positive_ranks[0].to_le_bytes()[0] >> 1) & 0b1_1111
        } & !(self.negative_ranks[0].to_le_bytes()[0] >> 1);
        (0..5).fold(0, |mask, suit| {
            mask | ((u32::from(suits & (1 << suit) != 0) * u32::from(ranks)) << (suit * 5))
        })
    }

    /// Direct physical clue mask in the active universe's identity order.
    /// Each positive clue must touch; every negative clue must miss.
    #[must_use]
    pub fn identity_mask_in(self, variant: &VariantSpec) -> u64 {
        variant
            .identities()
            .identities()
            .iter()
            .fold(0, |mask, identity| {
                let colors_match = |value| {
                    variant.touches(identity.id, GenericClue::Color(ClueColorId::new(value)))
                };
                let ranks_match =
                    |value| variant.touches(identity.id, GenericClue::Rank(ClueRank::new(value)));
                let allows = Self::all_match(self.positive_suits, colors_match)
                    && Self::all_match(self.negative_suits, |value| !colors_match(value))
                    && Self::all_match(self.positive_ranks, ranks_match)
                    && Self::all_match(self.negative_ranks, |value| !ranks_match(value));
                mask | (u64::from(allows) << identity.id.value())
            })
    }

    /// Records a direct positive standard clue fact.
    pub fn add_positive_clue(&mut self, clue: Clue) {
        self.record(clue, true);
    }

    /// Records a direct negative standard clue fact.
    pub fn add_negative_clue(&mut self, clue: Clue) {
        self.record(clue, false);
    }

    /// Whether this card was positively touched by a standard clue.
    #[must_use]
    pub fn has_positive_clue(self, clue: Clue) -> bool {
        self.has_positive_physical_clue(clue.into())
    }

    /// Whether this card was negatively excluded by a standard clue.
    #[must_use]
    pub fn has_negative_clue(self, clue: Clue) -> bool {
        self.has_negative_physical_clue(clue.into())
    }

    /// Whether this card was positively touched by a physical clue.
    #[must_use]
    pub fn has_positive_physical_clue(self, clue: GenericClue) -> bool {
        let (bits, value) = match clue {
            GenericClue::Color(color) => (self.positive_suits, color.value()),
            GenericClue::Rank(rank) => (self.positive_ranks, rank.value()),
        };
        Self::contains(bits, u64::from(value))
    }

    /// Whether this card was negatively excluded by a physical clue.
    #[must_use]
    pub fn has_negative_physical_clue(self, clue: GenericClue) -> bool {
        let (bits, value) = match clue {
            GenericClue::Color(color) => (self.negative_suits, color.value()),
            GenericClue::Rank(rank) => (self.negative_ranks, rank.value()),
        };
        Self::contains(bits, u64::from(value))
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        const fn empty(bits: [u64; 4]) -> bool {
            bits[0] | bits[1] | bits[2] | bits[3] == 0
        }
        empty(self.positive_suits)
            && empty(self.negative_suits)
            && empty(self.positive_ranks)
            && empty(self.negative_ranks)
    }

    pub(crate) fn record(&mut self, clue: Clue, positive: bool) {
        self.record_physical(clue.into(), positive);
    }

    /// Records a public physical clue fact without assigning a hidden identity.
    /// Callers validate the label against the active variant before recording it.
    pub fn record_physical(&mut self, clue: GenericClue, positive: bool) {
        let (bits, value) = match (clue, positive) {
            (GenericClue::Color(color), true) => (&mut self.positive_suits, color.value()),
            (GenericClue::Color(color), false) => (&mut self.negative_suits, color.value()),
            (GenericClue::Rank(rank), true) => (&mut self.positive_ranks, rank.value()),
            (GenericClue::Rank(rank), false) => (&mut self.negative_ranks, rank.value()),
        };
        bits[usize::from(value / 64)] |= 1 << (value % 64);
    }

    fn contains(bits: [u64; 4], value: u64) -> bool {
        bits[usize::try_from(value / 64).expect("u8 clue label")] & (1 << (value % 64)) != 0
    }

    fn all_match(bits: [u64; 4], matches: impl Fn(u8) -> bool) -> bool {
        for (index, mut word) in bits.into_iter().enumerate() {
            while word != 0 {
                let value = u8::try_from(index * 64 + word.trailing_zeros() as usize)
                    .expect("u8 clue label");
                if !matches(value) {
                    return false;
                }
                word &= word - 1;
            }
        }
        true
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
pub enum ObservedEvent<C = Clue> {
    Clued {
        giver: PlayerId,
        target: PlayerId,
        clue: C,
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
pub struct ObservedHistoryEntry<C = Clue> {
    pub turn: u32,
    pub event: ObservedEvent<C>,
}

/// Everything one player is legally permitted to observe.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PlayerView<C = Clue> {
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
    pub history: Vec<ObservedHistoryEntry<C>>,
}

impl<C> ObservedEvent<C> {
    /// Maps only the public clue label, preserving identities and observer redaction.
    pub(crate) fn map_clue<D>(self, map: impl FnOnce(C) -> D) -> ObservedEvent<D> {
        match self {
            Self::Clued {
                giver,
                target,
                clue,
                touched,
                untouched,
            } => ObservedEvent::Clued {
                giver,
                target,
                clue: map(clue),
                touched,
                untouched,
            },
            Self::Played {
                player,
                card,
                identity,
                successful,
            } => ObservedEvent::Played {
                player,
                card,
                identity,
                successful,
            },
            Self::Discarded {
                player,
                card,
                identity,
            } => ObservedEvent::Discarded {
                player,
                card,
                identity,
            },
            Self::Drew {
                player,
                card,
                identity,
            } => ObservedEvent::Drew {
                player,
                card,
                identity,
            },
        }
    }
}

impl PlayerView<GenericClue> {
    /// Converts public clue labels to the legacy No Variant domain.
    /// Physical meaning is equivalent for No Variant only.
    ///
    /// # Errors
    ///
    /// Returns an error if any clue cannot be represented without losing its label.
    ///
    /// # Panics
    ///
    /// Panics only if a previously checked clue conversion becomes inconsistent.
    pub fn try_standard(self) -> Result<PlayerView, StandardClueError> {
        let history = self
            .history
            .into_iter()
            .map(|entry| {
                if let ObservedEvent::Clued { clue, .. } = &entry.event {
                    Clue::try_from(*clue)?;
                }
                Ok(ObservedHistoryEntry {
                    turn: entry.turn,
                    event: entry
                        .event
                        .map_clue(|clue| Clue::try_from(clue).expect("checked clue domain")),
                })
            })
            .collect::<Result<Vec<_>, StandardClueError>>()?;
        Ok(PlayerView {
            variant: self.variant,
            observer: self.observer,
            current_player: self.current_player,
            turn: self.turn,
            hands: self.hands,
            deck_size: self.deck_size,
            play_stacks: self.play_stacks,
            discard_pile: self.discard_pile,
            clue_tokens: self.clue_tokens,
            strikes: self.strikes,
            final_turns_remaining: self.final_turns_remaining,
            status: self.status,
            history,
        })
    }
}

impl<C> PlayerView<C> {
    /// Legal physical actions, using only visible target identities and the
    /// public variant. Returns no actions for a nonacting observer or ended game.
    ///
    /// # Panics
    ///
    /// Panics only if a core-constructed view violates its player-count invariant.
    #[must_use]
    pub fn legal_physical_actions(&self) -> Vec<PhysicalAction> {
        if self.observer != self.current_player || self.status != GameStatus::InProgress {
            return Vec::new();
        }
        let hand = &self.hands[self.observer.index()];
        let mut actions = Vec::new();
        actions.extend(hand.iter().map(|card| PhysicalAction::Play(card.id)));
        if self.clue_tokens < MAX_CLUE_TOKENS {
            actions.extend(hand.iter().map(|card| PhysicalAction::Discard(card.id)));
        }
        if self.clue_tokens > 0 {
            for (index, hand) in self.hands.iter().enumerate() {
                if index == self.observer.index() {
                    continue;
                }
                let target = PlayerId::new(u8::try_from(index).expect("validated player count"));
                let identities = hand
                    .iter()
                    .filter_map(|card| card.identity)
                    .filter_map(|card| self.variant.identities().id_for(card.suit, card.rank))
                    .collect::<Vec<_>>();
                actions.extend(
                    self.variant
                        .legal_clues(&identities)
                        .into_iter()
                        .map(|clue| PhysicalAction::Clue { target, clue }),
                );
            }
        }
        actions
    }
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
    /// Projects state using legacy No Variant clue labels. Returns `None` for
    /// an invalid observer or a history containing an unrepresentable clue.
    /// Use [`Self::physical_view_for`] for arbitrary physical variants.
    ///
    /// # Panics
    ///
    /// Panics only if the full state violates its internal card-location
    /// invariant.
    #[must_use]
    pub fn view_for(&self, observer: PlayerId) -> Option<PlayerView> {
        self.physical_view_for(observer)?.try_standard().ok()
    }

    /// Projects authoritative history with generic physical clue labels.
    /// Own-hand and own-draw identities remain hidden.
    ///
    /// # Panics
    ///
    /// Panics only if a full state violates its card-location invariant.
    #[must_use]
    pub fn physical_view_for(&self, observer: PlayerId) -> Option<PlayerView<GenericClue>> {
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

fn observed_event(
    state: &FullState,
    observer: PlayerId,
    event: &GameEvent<GenericClue>,
) -> ObservedEvent<GenericClue> {
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
