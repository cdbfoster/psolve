use std::cmp::Ordering;

use crate::card::Card;

pub type Hand<const N: usize> = [Card; N];

pub trait HandComparator<const N: usize> {
    type HandRank: Ord;

    fn hand_rank(&self, hand: &Hand<N>) -> Self::HandRank;

    fn compare_hands(&self, a: &Hand<N>, b: &Hand<N>) -> Option<Ordering> {
        self.hand_rank(a).partial_cmp(&self.hand_rank(b))
    }
}
