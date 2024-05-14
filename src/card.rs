#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Suit {
    H,
    D,
    C,
    S,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Card {
    Shaped(Suit, u8),
    Joker,
}

impl Card {
    pub fn is_mighty(&self, trump: Option<Suit>) -> bool {
        match self {
            Card::Shaped(suit, n) => match trump {
                Some(Suit::S) => suit == &Suit::D && *n == 14,
                _ => suit == &Suit::S && *n == 14,
            },
            Card::Joker => false,
        }
    }

    pub fn deal_score(&self, trump: Option<Suit>) -> u8 {
        if self.is_mighty(trump) {
            0
        } else {
            match self {
                Card::Shaped(_, n) => {
                    if *n >= 10 {
                        1
                    } else {
                        0
                    }
                }
                Card::Joker => 0,
            }
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            Card::Shaped(_, n) => {
                if *n >= 10 {
                    1
                } else {
                    0
                }
            }
            Card::Joker => 0,
        }
    }

    pub fn is_joker_call(&self, trump: Option<Suit>) -> bool {
        match self {
            Card::Shaped(suit, n) => match trump {
                Some(Suit::C) => suit == &Suit::H && *n == 3,
                _ => suit == &Suit::C && *n == 3,
            },
            Card::Joker => false,
        }
    }
}
