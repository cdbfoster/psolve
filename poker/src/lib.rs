use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{self, Write};
use std::mem;

pub type Value = f32;

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

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Suit {
    Clubs = 0x10,
    Diamonds = 0x20,
    Hearts = 0x30,
    Spades = 0x40,
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

pub type Hand<const N: usize> = [Card; N];

pub trait HandComparator<const N: usize> {
    type HandRank: Ord;

    fn hand_rank(&self, hand: &Hand<N>) -> Self::HandRank;

    fn compare_hands(&self, a: &Hand<N>, b: &Hand<N>) -> Option<Ordering> {
        self.hand_rank(a).partial_cmp(&self.hand_rank(b))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Player(pub(crate) u8);

#[derive(Clone, Copy, Debug)]
pub struct State<T, const N: usize> {
    pub player_stacks: [Value; N],
    pub player_committed: [Value; N],
    pub player_folded: [bool; N],
    pub active_player: Player,
    pub last_aggressor: Option<Player>,

    pub pot: Value,
    pub last_raise: Value,
    pub current_bet: Value,

    pub game_data: T,
}

impl<T, const N: usize> State<T, N> {
    pub fn active_player_count(&self) -> usize {
        self.player_folded.iter().filter(|&&p| !p).count()
    }

    pub fn next_active_player(&self) -> Option<Player> {
        std::iter::repeat(0..N)
            .take(2)
            .flatten()
            .skip(self.active_player.0 as usize + 1)
            .find(|&p| !self.player_folded[p])
            .map(|p| Player(p as u8))
    }
}
