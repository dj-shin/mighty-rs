use crate::card::{Card, Suit};
use crate::common::{Contract, Hand, PlayerIndex, MAX_EFFECTIVE_COUNT};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct BiddingState {
    pub hand: Hand,
    pub curr_contract: Option<Contract>,
    // TODO: consider other players' contracts
}

#[derive(Clone, Debug)]
pub struct BiddingPhaseState {
    pub hands: Vec<Hand>,
    pub curr_player: PlayerIndex,
    pub bidding: Vec<Option<Contract>>,
}

#[derive(Clone, Debug)]
pub struct PledgePhase {
    pub hands: Vec<Hand>,
    pub curr_contract: Option<Contract>,
    pub call_history: Vec<(PlayerIndex, Option<Contract>)>,
    pub players_queue: Vec<PlayerIndex>,
    pub bottom: HashSet<Card>,

    pub min_effective_count: u8,
}

impl PledgePhase {
    pub fn new(start_player: PlayerIndex, min_pledge: u8) -> Self {
        let mut cards = vec![Card::Joker];
        for suit in [Suit::H, Suit::D, Suit::C, Suit::S] {
            for n in 2..=14 {
                cards.push(Card::Shaped(suit, n));
            }
        }
        cards.shuffle(&mut thread_rng());
        let mut hands: Vec<Hand> = vec![];
        for _ in 0..5 {
            hands.push(cards.drain(0..10).collect());
        }
        let bottom = cards.iter().cloned().collect();
        let mut players_queue = (0..5).collect::<Vec<PlayerIndex>>();
        players_queue.rotate_left(start_player);
        PledgePhase {
            hands,
            curr_contract: None,
            call_history: vec![],
            players_queue,
            bottom,
            min_effective_count: min_pledge - 1, // -1 for no suit
        }
    }

    pub fn bidding_state(&self, player: PlayerIndex) -> BiddingState {
        BiddingState {
            hand: self.hands[player].clone(),
            curr_contract: self.curr_contract,
        }
    }

    pub fn player_bids(&mut self, player: PlayerIndex, pledge: Option<Contract>) {
        assert_eq!(player, self.players_queue[0]);
        match pledge {
            Some(contract) => {
                assert!(contract.effective_count() > self.min_effective_count);
                self.min_effective_count = contract.effective_count();
                self.curr_contract = Some(contract);
                self.players_queue.rotate_left(1);
            }
            None => {
                self.players_queue.remove(0);
            }
        }
        self.call_history.push((player, pledge));
    }

    pub fn pledge_done(&self) -> bool {
        match self.curr_contract {
            Some(curr_contract) => {
                self.players_queue.len() <= 1
                    || curr_contract.effective_count() >= MAX_EFFECTIVE_COUNT
            }
            None => self.players_queue.len() == 0,
        }
    }

    pub fn turn_player(&self) -> PlayerIndex {
        self.players_queue[0]
    }

    pub fn cancelled(&self) -> bool {
        self.players_queue.len() == 0
    }
}
