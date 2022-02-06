use std::fmt;
use std::mem;

use util::math::ncr;

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

#[derive(Clone, Copy, Debug)]
pub struct CardRange {
    len: u8,
    start: u8,
    end: u8,
    /// Bitmap
    present: u64,
}

impl CardRange {
    pub fn from_cards(cards: &[Card]) -> Self {
        let start = card_index(*cards.iter().min().unwrap());
        let end = card_index(*cards.iter().max().unwrap());

        let mut range = Self {
            len: cards.len() as u8,
            start,
            end,
            present: 0,
        };

        for &card in cards {
            assert!(range.add(card), "duplicate card in deck");
        }

        range
    }

    pub(crate) fn from_parts(start: Card, end: Card, present: u64) -> Self {
        Self {
            len: present.count_ones() as u8,
            start: card_index(start),
            end: card_index(end),
            present,
        }
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn contains(&self, card: Card) -> bool {
        self.present & 1u64 << (card_index(card) - self.start) == 1
    }

    /// Returns true if the card was successfully added to the range.
    /// False if out of range, or already present.
    pub fn add(&mut self, card: Card) -> bool {
        let i = card_index(card);
        if i >= self.start && i <= self.end {
            let mask = 1u64 << (i - self.start);
            let exists = self.present & mask == 1;
            self.present |= mask;
            !exists
        } else {
            false
        }
    }

    /// Returns true if the card was successfully removed from the range.
    /// False if out of range, or already missing.
    pub fn remove(&mut self, card: Card) -> bool {
        let i = card_index(card);
        if i >= self.start && i <= self.end {
            let mask = 1u64 << (i - self.start);
            let exists = self.present & mask == 1;
            self.present &= !mask;
            exists
        } else {
            false
        }
    }

    /// None if the card is not present in the range.
    pub fn index_of(&self, card: Card) -> Option<usize> {
        let i = card_index(card);

        (i >= self.start && i <= self.end && self.present & 1u64 << i == 1).then(|| {
            let mask = 0xFFFFFFFFFFFFFFFF >> (63 - i);
            let offset = (self.present & mask).count_zeros() as usize;
            i as usize - offset
        })
    }

    /// Assumes cards have been sorted into descending order.
    /// None if any of the cards are not present in the range.
    pub fn combo_index_of(&self, cards: &[Card]) -> Option<usize> {
        let mut sum = 0;

        for (i, &c) in cards.iter().enumerate() {
            if let Some(x) = self.index_of(c) {
                sum += ncr(x, cards.len() - i);
            } else {
                return None;
            }
        }

        Some(sum)
    }
}

fn card_index(card: Card) -> u8 {
    ((card.suit() as u8 >> 4) - 1) * 13 + (card.rank() as u8 - 2)
}
