use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Write};
use std::mem;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn next(self) -> Option<Self> {
        (self as u8 + 1).try_into().ok()
    }

    pub fn previous(self) -> Option<Rank> {
        (self as u8 - 1).try_into().ok()
    }
}

impl fmt::Debug for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        })
    }
}

impl TryFrom<u8> for Rank {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value >= 2 && value <= 14 {
            Ok(unsafe { mem::transmute(value) })
        } else {
            Err("invalid card rank")
        }
    }
}
