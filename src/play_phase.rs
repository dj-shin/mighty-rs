use crate::card::{Card, Suit};
use crate::common::{Contract, Hand, PartnerCondition, PlayAction, PlayerIndex};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RoundResult {
    pub winner: PlayerIndex,
    pub submitted: Vec<Card>,
}

#[derive(Clone, Debug)]
pub struct ExposedGameState {
    // Static state
    pub hand: Hand,
    pub declarer: PlayerIndex,
    pub contract: Contract,
    pub partner_condition: PartnerCondition,
    pub discarded: Option<HashSet<Card>>,

    // Dynamic state
    pub partner_revealed: Option<PlayerIndex>,

    // Round state
    pub round: u8,
    pub joker_called: bool,
    pub submitted: Vec<Option<Card>>,
    pub round_starter: PlayerIndex,
    pub round_suit: Option<Suit>,

    // History
    pub round_results: Vec<RoundResult>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct PlayPhase {
    // Static state
    pub hands: Vec<Hand>,
    pub declarer: PlayerIndex,
    pub contract: Contract,
    pub partner_condition: PartnerCondition,
    pub discarded: HashSet<Card>,

    // Dynamic state
    pub partner_revealed: Option<PlayerIndex>,

    // Round state
    pub round: u8,
    pub joker_called: bool,
    pub submitted: Vec<Option<Card>>,
    pub round_starter: PlayerIndex,
    pub round_suit: Option<Suit>,

    // History
    pub round_results: Vec<RoundResult>,
}

impl fmt::Debug for PlayPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PlayPhase [
  hands: [
    p1: {:?}
    p2: {:?}
    p3: {:?}
    p4: {:?}
    p5: {:?}
  ]
  submitted: {:?}
  history: {:#?}
]",
            self.hands[0],
            self.hands[1],
            self.hands[2],
            self.hands[3],
            self.hands[4],
            self.submitted,
            self.round_results,
        )
    }
}

impl PlayPhase {
    pub fn current_round_order(&self) -> Vec<PlayerIndex> {
        let mut players_queue = (0..5).collect::<Vec<PlayerIndex>>();
        players_queue.rotate_left(self.round_starter);
        return players_queue;
    }

    pub fn play_state(&self, player_index: PlayerIndex) -> ExposedGameState {
        let discarded = if player_index == self.declarer {
            Some(self.discarded.clone())
        } else {
            None
        };
        let hand = self.hands[player_index].clone();
        let partner_revealed = self.partner_revealed;

        ExposedGameState {
            hand,
            declarer: self.declarer,
            contract: self.contract,
            partner_condition: self.partner_condition,
            discarded,
            partner_revealed,
            round: self.round,
            joker_called: self.joker_called,
            submitted: self.submitted.clone(),
            round_starter: self.round_starter,
            round_suit: self.round_suit,
            round_results: self.round_results.clone(),
        }
    }

    fn is_player_partner(&self, player_index: PlayerIndex) -> bool {
        match self.partner_condition {
            PartnerCondition::CardCondition(card) => self.hands[player_index].contains(&card),
            PartnerCondition::Round(n) => self
                .round_results
                .get(n as usize)
                .map_or(false, |round_result| round_result.winner == player_index),
            PartnerCondition::Player(p) => p == player_index,
            PartnerCondition::None => false,
        }
    }

    pub fn player_acts(&mut self, player_index: PlayerIndex, action: PlayAction) {
        match action {
            PlayAction::Hand(card) => {
                match card {
                    Card::Shaped(s, _) => {
                        let hand = &self.hands[player_index];

                        assert!(hand.contains(&card));
                        if !card.is_mighty(self.contract.suit)
                            && self.round_suit.is_some_and(|round_suit| {
                                hand.iter().any(|&c| {
                                    if let Card::Shaped(s, _) = c {
                                        s == round_suit
                                    } else {
                                        false
                                    }
                                })
                            })
                        {
                            assert_eq!(self.round_suit.unwrap(), s);
                        }

                        if player_index == self.round_starter {
                            // Round starter
                            self.round_suit = Some(s);
                        }
                    }
                    Card::Joker => {
                        assert_ne!(player_index, self.round_starter);
                    }
                }
                self.submitted[player_index] = Some(card);
                self.hands[player_index].remove(&card);
            }
            PlayAction::JokerCall(card) => {
                assert_eq!(player_index, self.round_starter);
                assert!(card.is_joker_call(self.contract.suit));
                assert!(self.hands[player_index].contains(&card));

                self.joker_called = true;
                self.submitted[player_index] = Some(card);
                self.hands[player_index].remove(&card);
            }
            PlayAction::JokerStart(s) => {
                assert_eq!(player_index, self.round_starter);
                assert!(self.hands[player_index].contains(&Card::Joker));
                self.hands[player_index].remove(&Card::Joker);
                self.submitted[player_index] = Some(Card::Joker);
                self.round_suit = Some(s);
            }
        }

        if player_index == (self.round_starter + 4) % 5 {
            // Round last
            let winner = self.round_winner();
            self.round_results.push(RoundResult {
                winner,
                submitted: self.submitted.iter().map(|v| v.unwrap()).collect(),
            });

            self.round += 1;
            self.round_suit = None;
            self.submitted = vec![None; 5];
            self.round_starter = winner;
            self.joker_called = false;
        }
    }

    fn round_winner(&self) -> PlayerIndex {
        self.submitted
            .iter()
            .enumerate()
            .max_by_key(|&(_, &c)| self.card_value(c.unwrap()))
            .map(|(i, _)| i)
            .unwrap()
    }

    fn card_value(&self, card: Card) -> u8 {
        return match card {
            Card::Joker => {
                if self.joker_called || self.round == 0 || self.round == 9 {
                    0
                } else {
                    100
                }
            }
            Card::Shaped(s, n) => {
                if card.is_mighty(self.contract.suit) {
                    200
                } else if self.contract.suit == Some(s) {
                    70 + n
                } else if self.round_suit == Some(s) {
                    30 + n
                } else {
                    n
                }
            }
        };
    }
}
