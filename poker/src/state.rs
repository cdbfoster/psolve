pub type Value = u32;

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
