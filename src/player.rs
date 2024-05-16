use crate::bidding_phase::BiddingState;
use crate::card::Card;
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
        (state.contract, PartnerCondition::None, discard)
    }

    fn play_action(&self, state: ExposedGameState) -> PlayAction {
        let random_card = state.hand.iter().next().unwrap().clone();
        PlayAction::Hand(random_card)
    }
}
