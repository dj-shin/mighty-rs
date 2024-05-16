use crate::card::Card;
use crate::common::{Contract, Hand, PartnerCondition, PlayAction, PlayerIndex};
use std::collections::HashSet;

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
    pub partner_revealed: Option<PlayerIndex>,

    // Round state
    pub round: u8,
    pub joker_called: bool,
    pub submitted: Vec<Option<Card>>,
    pub round_starter: PlayerIndex,

    // History
    pub round_results: Vec<RoundResult>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PlayPhase {
    // Static state
    pub hands: Vec<Hand>,
    pub declarer: PlayerIndex,
    pub contract: Contract,
    pub partner_condition: PartnerCondition,
    pub discarded: HashSet<Card>,
    pub partner_revealed: Option<PlayerIndex>,

    // Round state
    pub round: u8,
    pub joker_called: bool,
    pub submitted: Vec<Option<Card>>,
    pub round_starter: PlayerIndex,

    // History
    pub round_results: Vec<RoundResult>,
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
                self.hands[player_index].remove(&card);
            }
            PlayAction::JokerCall => {}
            PlayAction::JokerStart(_) => {}
        }
    }
}
