use crate::card::{Card, Suit};
use std::collections::HashSet;

pub type Hand = HashSet<Card>;
pub type PlayerIndex = usize;

pub const MAX_EFFECTIVE_COUNT: u8 = 21; // 풀 노기루

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PartnerCondition {
    CardCondition(Card),
    Round(u8),
    Player(PlayerIndex),
    None,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PlayAction {
    Hand(Card),
    JokerCall,
    JokerStart(Suit),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Contract {
    pub suit: Option<Suit>,
    pub count: u8,
}

impl Contract {
    pub fn effective_count(&self) -> u8 {
        match self.suit {
            Some(_) => self.count,
            None => self.count + 1,
        }
    }
}
