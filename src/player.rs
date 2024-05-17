use crate::bidding_phase::BiddingState;
use crate::card::{Card, Suit};
use crate::common::{Contract, PartnerCondition, PlayAction};
use crate::extra_phase::ExtraExposedState;
use crate::play_phase::ExposedGameState;
use std::collections::HashSet;

pub trait Player {
    fn bidding(&self, state: &BiddingState) -> Option<Contract>;
    fn declare_plan(&self, state: ExtraExposedState)
        -> (Contract, PartnerCondition, HashSet<Card>);
    fn play_action(&self, state: ExposedGameState) -> PlayAction;
}

pub struct RandomPlayer {}

impl RandomPlayer {
    fn playable_cards(&self, state: &ExposedGameState) -> HashSet<Card> {
        if state.joker_called && state.hand.contains(&Card::Joker) {
            HashSet::from([Card::Joker])
        } else {
            match state.round_suit {
                None => state.hand.clone(),
                Some(round_suit) => {
                    let has_suit = state.hand.iter().any(|&c| {
                        if let Card::Shaped(s, _) = c {
                            s == round_suit
                        } else {
                            false
                        }
                    });
                    if has_suit {
                        state
                            .hand
                            .iter()
                            .filter(|&c| {
                                if c.is_mighty(state.contract.suit) {
                                    true
                                } else {
                                    match c {
                                        Card::Shaped(s, _) => s == &round_suit,
                                        Card::Joker => true,
                                    }
                                }
                            })
                            .cloned()
                            .collect()
                    } else {
                        state.hand.clone()
                    }
                }
            }
        }
    }
}

impl Player for RandomPlayer {
    fn bidding(&self, state: &BiddingState) -> Option<Contract> {
        if state.curr_contract.is_none() {
            return Some(Contract {
                suit: None,
                count: 12,
            });
        }
        return None;
    }

    fn declare_plan(
        &self,
        state: ExtraExposedState,
    ) -> (Contract, PartnerCondition, HashSet<Card>) {
        let discard = state
            .hand
            .iter()
            .take(3)
            .cloned()
            .collect::<HashSet<Card>>();
        (
            state.contract,
            PartnerCondition::CardCondition(Card::Shaped(Suit::S, 14)),
            discard,
        )
    }

    fn play_action(&self, state: ExposedGameState) -> PlayAction {
        let random_card = self.playable_cards(&state).iter().next().unwrap().clone();
        match random_card {
            Card::Shaped(_, _) => PlayAction::Hand(random_card),
            Card::Joker => {
                if state.submitted.iter().all(|v| v.is_none()) {
                    PlayAction::JokerStart(Suit::S)
                } else {
                    PlayAction::Hand(random_card)
                }
            }
        }
    }
}
