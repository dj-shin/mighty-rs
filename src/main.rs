mod card;

use std::collections::HashSet;
use crate::card::{Card, Suit};
use rand::prelude::SliceRandom;
use rand::thread_rng;

type Hand = HashSet<Card>;
type PlayerIndex = usize;

const MAX_EFFECTIVE_COUNT: u8 = 21; // 풀 노기루

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum PartnerCondition {
    CardCondition(Card),
    Round(u8),
    Player(PlayerIndex),
    None,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum PlayAction {
    Hand(Card),
    JokerCall,
    JokerStart(Suit),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Contract {
    suit: Option<Suit>,
    count: u8,
}

impl Contract {
    pub fn effective_count(&self) -> u8 {
        match self.suit {
            Some(_) => self.count,
            None => self.count + 1,
        }
    }
}

#[derive(Clone, Debug)]
struct BiddingPhaseState {
    hands: Vec<Hand>,
    curr_player: PlayerIndex,
    bidding: Vec<Option<Contract>>,
}

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

#[derive(Clone, Debug)]
struct ExposedGameState {
    discarded: Option<HashSet<Card>>,
    prev_played: Vec<Vec<Card>>,
    curr_played: Vec<Card>,
    trump: Suit,
    declarer: PlayerIndex,
    partner: Option<PlayerIndex>,
    hand: Hand,
}

#[derive(Clone, Debug)]
struct BiddingState {
    hand: Hand,
    curr_contract: Option<Contract>,
    // TODO: consider other players' contracts
}

#[derive(Clone, Debug)]
struct ExtraExposedState {
    hand: Hand,
    contract: Contract,
    // TODO: consider other players' contracts
}

#[derive(Clone, Debug)]
struct PledgePhase {
    hands: Vec<Hand>,
    curr_contract: Option<Contract>,
    call_history: Vec<(PlayerIndex, Option<Contract>)>,
    players_queue: Vec<PlayerIndex>,
    bottom: HashSet<Card>,

    min_effective_count: u8,
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
            min_effective_count: min_pledge - 1,    // -1 for no suit
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
            },
            None => {
                self.players_queue.remove(0);
            }
        }
        self.call_history.push((player, pledge));
    }

    pub fn pledge_done(&self) -> bool {
        match self.curr_contract {
            Some(curr_contract) => {
                self.players_queue.len() <= 1 || curr_contract.effective_count() >= MAX_EFFECTIVE_COUNT
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

trait Player {
    fn next_hand(&self, state: &ExposedGameState) -> Card;
    fn bidding(&self, state: &BiddingState) -> Option<Contract>;
    fn declare_plan(&self, state: ExtraExposedState) -> (Contract, PartnerCondition, HashSet<Card>);
    fn play_action(&self, state: ExposedGameState) -> PlayAction;
}

struct RandomPlayer {}

impl Player for RandomPlayer {
    fn next_hand(&self, state: &ExposedGameState) -> Card {
        unimplemented!()
    }

    fn bidding(&self, state: &BiddingState) -> Option<Contract> {
        if state.curr_contract.is_none() {
            return Some(Contract {
                suit: None,
                count: 12,
            })
        }
        return None;
    }

    fn declare_plan(&self, state: ExtraExposedState) -> (Contract, PartnerCondition, HashSet<Card>) {
        let discard = state.hand.iter().take(3).cloned().collect::<HashSet<Card>>();
        (
            state.contract,
            PartnerCondition::None,
            discard,
        )
    }

    fn play_action(&self, state: ExposedGameState) -> PlayAction {
        unimplemented!()
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct ExtraPhase {
    hands: Vec<Hand>,
    contract: Contract,
    declarer: PlayerIndex,
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
        PlayPhase {
            hands: self.hands.clone(),
            declarer: self.declarer,
            contract,
            partner_condition,
            discarded: discards,
            partner_revealed: None,
            round: 0,
            joker_called: false,
            submitted: vec![],
            round_results: vec![],
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct RoundResult {
    winner: PlayerIndex,
    submitted: Vec<Card>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct PlayPhase {
    // Static state
    hands: Vec<Hand>,
    declarer: PlayerIndex,
    contract: Contract,
    partner_condition: PartnerCondition,
    discarded: HashSet<Card>,
    partner_revealed: Option<PlayerIndex>,

    // Round state
    round: u8,
    joker_called: bool,
    submitted: Vec<Option<Card>>,

    // History
    round_results: Vec<RoundResult>,
}

impl PlayPhase {
    pub fn current_turn_order(&self) -> Vec<PlayerIndex> {
        unimplemented!()
    }

    pub fn play_state(&self, player_index: PlayerIndex) -> ExposedGameState {
        unimplemented!()
    }

    pub fn player_acts(&mut self, player_index: PlayerIndex, action: PlayAction) {
        unimplemented!()
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
    let declarer_index = game.declarer();
    let declarer = &players[declarer_index];

    println!("주공: Player {}", declarer_index);
    let (contract, condition, discards) = declarer.declare_plan(game.extra_state());
    println!("공약: {:?}", condition);
    let mut game = game.submit_plan(contract, condition, discards);

    println!("== 플레이 ==");
    for round in 0..10 {
        println!("== Round {} ==", round + 1);
        for player_index in game.current_turn_order() {
            let player = &players[player_index];
            let curr_state = game.play_state(player_index);
            let action = player.play_action(curr_state);
            println!("Player {} : {:?}", player_index, action);
            game.player_acts(player_index, action);
        }
    }
}
