use rand::seq::SliceRandom;
use rand::Rng;

use crate::card::Card;

#[derive(Clone, Copy, Debug)]
pub struct Deck<const N: usize> {
    len: u8,
    cards: [Card; N],
}

pub fn full_deck() -> Deck<52> {
    use Card::*;
    Deck {
        len: 52,
        #[rustfmt::skip]
        cards: [
            C2, C3, C4, C5, C6, C7, C8, C9, CT, CJ, CQ, CK, CA,
            D2, D3, D4, D5, D6, D7, D8, D9, DT, DJ, DQ, DK, DA,
            H2, H3, H4, H5, H6, H7, H8, H9, HT, HJ, HQ, HK, HA,
            S2, S3, S4, S5, S6, S7, S8, S9, ST, SJ, SQ, SK, SA,
        ],
    }
}

impl<const N: usize> Deck<N> {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn shuffle<R: Rng>(&mut self, rng: &mut R) {
        let l = self.len();
        self.cards[0..l].shuffle(rng);
    }

    pub fn deal_card(&mut self) -> Card {
        assert!(self.len() > 0, "not enough cards remaining");
        self.len -= 1;
        self.cards[self.len() + 1]
    }

    pub fn deal_cards(&mut self, cards: &mut [Card]) {
        assert!(self.len() >= cards.len(), "not enough cards remaining");
        cards.copy_from_slice(&self.cards[self.len() - cards.len()..self.len()]);
    }
}
