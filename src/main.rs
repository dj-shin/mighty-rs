mod bidding_phase;
mod card;
mod common;
mod extra_phase;
mod play_phase;
mod player;

use crate::bidding_phase::PledgePhase;
use crate::common::{Contract, Hand, PartnerCondition, PlayerIndex};
use crate::extra_phase::ExtraPhase;
use crate::player::{Player, RandomPlayer};
use rand::prelude::SliceRandom;

#[derive(Clone, Debug)]
struct GameState {
    hands: Vec<Hand>,
    contract: Contract,
    declarer: PlayerIndex,
    partner_condition: PartnerCondition,
    partner: Option<PlayerIndex>,
    round: u8,
    lead: PlayerIndex,

    curr_player: PlayerIndex,
}

impl GameState {
    fn new(
        hands: Vec<Hand>,
        contract: Contract,
        declarer: PlayerIndex,
        partner_condition: PartnerCondition,
    ) -> Self {
        GameState {
            hands,
            contract,
            declarer,
            partner_condition,
            partner: None,
            round: 1,
            lead: declarer,
            curr_player: declarer,
        }
    }
}

fn main() {
    let players: Vec<Box<dyn Player>> = vec![
        Box::new(RandomPlayer {}),
        Box::new(RandomPlayer {}),
        Box::new(RandomPlayer {}),
        Box::new(RandomPlayer {}),
        Box::new(RandomPlayer {}),
    ];

    let mut game = PledgePhase::new(0, 13);

    println!("== 공약 ==");
    while !game.pledge_done() {
        let player_index = game.turn_player();
        let player = &players[player_index];
        let bidding_state = game.bidding_state(player_index);
        let contract = player.bidding(&bidding_state);
        println!("Player {}: {:?}", player_index, contract);
        game.player_bids(player_index, contract);
    }
    if game.cancelled() {
        println!("Pledge cancelled");
        return;
    }

    let mut game = ExtraPhase::from_pledge(game);
    println!("{:?}", game);
    let declarer_index = game.declarer();
    let declarer = &players[declarer_index];

    println!("주공: Player {}", declarer_index);
    let (contract, condition, discards) = declarer.declare_plan(game.extra_state());
    println!("공약: {:?}", condition);

    println!("{:?}", discards);

    let mut game = game.submit_plan(contract, condition, discards);

    println!("{:?}", game);
    println!("== 플레이 ==");
    for round in 0..10 {
        println!("== Round {} ==", round + 1);
        for player_index in game.current_round_order() {
            let player = &players[player_index];
            let curr_state = game.play_state(player_index);
            let action = player.play_action(curr_state);
            println!("Player {} : {:?}", player_index, action);
            game.player_acts(player_index, action);
            println!("{:?}", game);
        }
    }
}
