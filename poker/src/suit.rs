use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Write};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Suit {
    Clubs = 0x10,
    Diamonds = 0x20,
    Hearts = 0x30,
    Spades = 0x40,
}

impl Suit {
    pub fn next(self) -> Option<Self> {
        (self as u8 + 0x10).try_into().ok()
    }

    pub fn previous(self) -> Option<Self> {
        (self as u8 - 0x10).try_into().ok()
    }
}

impl fmt::Debug for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Suit::Clubs => 'c',
            Suit::Diamonds => 'd',
            Suit::Hearts => 'h',
            Suit::Spades => 's',
        })
    }
}

impl TryFrom<u8> for Suit {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Suit::Clubs),
            0x20 => Ok(Suit::Diamonds),
            0x30 => Ok(Suit::Hearts),
            0x40 => Ok(Suit::Spades),
            _ => Err("invalid card suit"),
        }
    }
}
