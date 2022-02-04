use std::fmt;
use std::mem;

use crate::rank::Rank;
use crate::suit::Suit;

#[rustfmt::skip]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Card {
    C2 = 0x12, C3, C4, C5, C6, C7, C8, C9, CT, CJ, CQ, CK, CA,
    D2 = 0x22, D3, D4, D5, D6, D7, D8, D9, DT, DJ, DQ, DK, DA,
    H2 = 0x32, H3, H4, H5, H6, H7, H8, H9, HT, HJ, HQ, HK, HA,
    S2 = 0x42, S3, S4, S5, S6, S7, S8, S9, ST, SJ, SQ, SK, SA,
}

impl Card {
    pub fn from_rank_and_suit(rank: Rank, suit: Suit) -> Self {
        unsafe { mem::transmute(rank as u8 | suit as u8) }
    }

    pub fn rank(self) -> Rank {
        unsafe { mem::transmute(self as u8 & 0x0F) }
    }

    pub fn suit(self) -> Suit {
        unsafe { mem::transmute(self as u8 & 0xF0) }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.rank(), self.suit())
    }
}
