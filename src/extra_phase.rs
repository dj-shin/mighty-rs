use crate::bidding_phase::PledgePhase;
use crate::card::Card;
use crate::common::{Contract, Hand, PartnerCondition, PlayerIndex};
use crate::play_phase::PlayPhase;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct ExtraExposedState {
    pub hand: Hand,
    pub contract: Contract,
    // TODO: consider other players' contracts
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExtraPhase {
    pub hands: Vec<Hand>,
    pub contract: Contract,
    pub declarer: PlayerIndex,
}
impl ExtraPhase {
    pub fn from_pledge(game: PledgePhase) -> Self {
        let &declarer = game.players_queue.first().unwrap();
        let contract = game.curr_contract.unwrap();
        let bottom = game.bottom;
        let mut hands = game.hands;
        hands[declarer].extend(bottom);
        ExtraPhase {
            hands,
            declarer,
            contract,
        }
    }

    pub fn declarer(&self) -> PlayerIndex {
        self.declarer
    }

    pub fn extra_state(&self) -> ExtraExposedState {
        ExtraExposedState {
            hand: self.hands[self.declarer].clone(),
            contract: self.contract,
        }
    }

    pub fn submit_plan(
        &mut self,
        contract: Contract,
        partner_condition: PartnerCondition,
        discards: HashSet<Card>,
    ) -> PlayPhase {
        assert!(contract.effective_count() >= self.contract.effective_count());
        let declarer_hand = self.hands[self.declarer].clone();
        assert!(declarer_hand.is_superset(&discards));
        self.hands[self.declarer] = declarer_hand.difference(&discards).cloned().collect();

        PlayPhase {
            hands: self.hands.clone(),
            declarer: self.declarer,
            contract,
            partner_condition,
            discarded: discards,
            partner_revealed: None,
            round: 0,
            joker_called: false,
            submitted: vec![None; 5],
            round_results: vec![],
            round_starter: self.declarer,
        }
    }
}
