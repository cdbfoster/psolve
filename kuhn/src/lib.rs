use rand::{self, Rng};

use game_tree::{Event, Game, ParameterMapping, Stage};

#[derive(Clone, Copy, Debug)]
pub enum KuhnStage {
    PlayerAction(u8),
    Showdown,
}

impl Stage for KuhnStage {
    fn is_action(&self) -> bool {
        match self {
            KuhnStage::PlayerAction(_) => true,
            KuhnStage::Showdown => false,
        }
    }

    fn is_chance(&self) -> bool {
        false
    }

    fn is_terminal(&self) -> bool {
        match self {
            KuhnStage::PlayerAction(_) => false,
            KuhnStage::Showdown => true,
        }
    }

    fn player_to_act(&self) -> Option<usize> {
        match self {
            KuhnStage::PlayerAction(p) => Some(*p as usize),
            KuhnStage::Showdown => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KuhnAction {
    Bet,   // Also call if not the first bettor.
    Check, // Also fold if not the first bettor.
}

#[derive(Clone, Copy, Debug)]
pub struct KuhnState<const N: usize> {
    cards: [u8; N],
    bet: bool,
    called: [bool; N],
    stage: KuhnStage,
}

impl<const N: usize> KuhnState<N> {
    pub fn from_cards(cards: [u8; N]) -> Self {
        Self {
            cards,
            bet: false,
            called: [false; N],
            stage: KuhnStage::PlayerAction(0),
        }
    }

    pub fn random<R: Rng>(rng: &mut R) -> Self {
        let mut cards = [0u8; N];

        for (c, d) in cards
            .iter_mut()
            .zip(rand::seq::index::sample(rng, N + 1, N))
        {
            *c = d as u8;
        }

        Self::from_cards(cards)
    }
}

pub struct KuhnGame<const N: usize>;

impl<const N: usize> Game for KuhnGame<N> {
    type Action = KuhnAction;
    type Chance = ();
    type ParameterMapping = KuhnParameterMapping<N>;
    type Stage = KuhnStage;
    type State = KuhnState<N>;

    fn advance_state(state: &mut Self::State, event: Event<Self::Action, Self::Chance>) {
        if let Event::Action(action) = event {
            if let KuhnStage::PlayerAction(player) = state.stage {
                let next_player = (player + 1) as usize % N;

                if state.bet {
                    if matches!(action, KuhnAction::Bet) {
                        // This is a call.
                        state.called[player as usize] = true;
                    }

                    if state.called[next_player] {
                        state.stage = KuhnStage::Showdown;
                    } else {
                        state.stage = KuhnStage::PlayerAction(next_player as u8);
                    }
                } else if matches!(action, KuhnAction::Bet) {
                    // This is a bet.
                    state.bet = true;
                    state.called[player as usize] = true;
                    state.stage = KuhnStage::PlayerAction(next_player as u8);
                } else if next_player < player as usize {
                    // This is a check, as the last player to act.
                    state.stage = KuhnStage::Showdown;
                } else {
                    // This is a check, with more players to act after.
                    state.stage = KuhnStage::PlayerAction(next_player as u8);
                }
            } else {
                panic!("cannot advance a state that is at showdown");
            }
        } else {
            panic!("there are no chance events in kuhn poker");
        }
    }

    fn populate_events(state: &Self::State, events: &mut Vec<Event<Self::Action, Self::Chance>>) {
        events.clear();

        if !matches!(state.stage, KuhnStage::Showdown) {
            events.push(Event::Action(KuhnAction::Bet));
            events.push(Event::Action(KuhnAction::Check));
        }
    }

    fn get_chance_weight(_state: &Self::State, _event: Self::Chance) -> f32 {
        panic!("there are no chance events in kuhn poker")
    }

    fn sample_chance<R: Rng>(_state: &Self::State, _rng: &mut R) -> (Self::Chance, usize) {
        panic!("there are no chance events in kuhn poker")
    }

    fn get_stage(state: &Self::State) -> Self::Stage {
        state.stage
    }

    fn get_branching_hint(state: &Self::State) -> usize {
        match state.stage {
            KuhnStage::PlayerAction(_) => 2,
            KuhnStage::Showdown => 0,
        }
    }

    fn get_terminal_utilities(state: &Self::State, utilities: &mut [f32]) {
        assert!(
            matches!(state.stage, KuhnStage::Showdown),
            "stage must be showdown to calculate utility"
        );
        assert_eq!(utilities.len(), N, "utility array is the wrong length");

        let pot = N + state.called.iter().filter(|&&b| b).count();

        let winner = if state.bet {
            state
                .cards
                .iter()
                .enumerate()
                .filter(|(i, _)| state.called[*i])
                .max_by_key(|(_, &c)| c)
                .map(|(i, _)| i)
                .unwrap()
        } else {
            state
                .cards
                .iter()
                .enumerate()
                .max_by_key(|(_, &c)| c)
                .map(|(i, _)| i)
                .unwrap()
        };

        utilities.iter_mut().enumerate().for_each(|(i, u)| {
            if i == winner {
                *u = if state.called[i] {
                    pot as f32 - 2.0
                } else {
                    pot as f32 - 1.0
                }
            } else {
                *u = if state.called[i] { -2.0 } else { -1.0 }
            }
        });
    }
}

pub struct KuhnParameterMapping<const N: usize>;

impl<const N: usize> ParameterMapping for KuhnParameterMapping<N> {
    type State = KuhnState<N>;

    fn get_parameter_count(_state: &Self::State) -> usize {
        N + 1
    }

    fn get_parameter_index(state: &Self::State) -> usize {
        if let KuhnStage::PlayerAction(player) = state.stage {
            state.cards[player as usize] as usize
        } else {
            panic!("no parameter index for a non-player action stage")
        }
    }

    fn get_parameter_description(state: &Self::State, alternate_index: Option<usize>) -> String {
        assert!(N < 13, "too many players to describe");

        let index = if let KuhnStage::PlayerAction(player) = state.stage {
            if let Some(index) = alternate_index {
                index
            } else {
                state.cards[player as usize] as usize
            }
        } else {
            panic!("no parameter index for a non-player action stage");
        };

        assert!(index < N + 1, "parameter index is out of bounds");

        const CARDS: [char; 13] = [
            'A', '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K',
        ];

        let card_index = 12 - N + index;
        CARDS[card_index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use rand::SeedableRng;

    use game_tree::{allocate_tree, TreeEstimator};
    use solver::{dump_strategy, Cfr, CfrParameter, Solver};
    use util::arena::Arena;
    use util::rng::JKiss32Rng;

    #[test]
    fn test_kuhn_utilities() {
        const COMBOS: [[u8; 2]; 6] = [[0, 1], [0, 2], [1, 0], [1, 2], [2, 0], [2, 1]];

        // check-check

        let utilities = [
            [-1.0, 1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [1.0, -1.0],
        ];

        for (combo, expected) in COMBOS.iter().zip(utilities.iter()) {
            let state = KuhnState {
                cards: *combo,
                bet: false,
                called: [false; 2],
                stage: KuhnStage::Showdown,
            };

            let mut result = [0.0; 2];
            KuhnGame::get_terminal_utilities(&state, &mut result);

            assert_eq!(*expected, result, "incorrect check-check utilities",);
        }

        // check-bet-fold

        let utilities = [
            [-1.0, 1.0],
            [-1.0, 1.0],
            [-1.0, 1.0],
            [-1.0, 1.0],
            [-1.0, 1.0],
            [-1.0, 1.0],
        ];

        for (combo, expected) in COMBOS.iter().zip(utilities.iter()) {
            let state = KuhnState {
                cards: *combo,
                bet: true,
                called: [false, true],
                stage: KuhnStage::Showdown,
            };

            let mut result = [0.0; 2];
            KuhnGame::get_terminal_utilities(&state, &mut result);

            assert_eq!(*expected, result, "incorrect check-bet-fold utilities",);
        }

        // bet-fold

        let utilities = [
            [1.0, -1.0],
            [1.0, -1.0],
            [1.0, -1.0],
            [1.0, -1.0],
            [1.0, -1.0],
            [1.0, -1.0],
        ];

        for (combo, expected) in COMBOS.iter().zip(utilities.iter()) {
            let state = KuhnState {
                cards: *combo,
                bet: true,
                called: [true, false],
                stage: KuhnStage::Showdown,
            };

            let mut result = [0.0; 2];
            KuhnGame::get_terminal_utilities(&state, &mut result);

            assert_eq!(*expected, result, "incorrect bet-fold utilities",);
        }

        // bet-call and check-bet-call

        let utilities = [
            [-2.0, 2.0],
            [-2.0, 2.0],
            [2.0, -2.0],
            [-2.0, 2.0],
            [2.0, -2.0],
            [2.0, -2.0],
        ];

        for (combo, expected) in COMBOS.iter().zip(utilities.iter()) {
            let state = KuhnState {
                cards: *combo,
                bet: true,
                called: [true; 2],
                stage: KuhnStage::Showdown,
            };

            let mut result = [0.0; 2];
            KuhnGame::get_terminal_utilities(&state, &mut result);

            assert_eq!(*expected, result, "incorrect bet-call utilities",);
        }
    }

    #[test]
    fn test_tree_estimate() {
        const N: usize = 3;
        let root_state = KuhnState::from_cards([0; N]);

        let estimator = TreeEstimator::<KuhnGame<N>, CfrParameter>::from_root(root_state);

        assert_eq!(estimator.action_nodes(), 24);
        assert_eq!(estimator.chance_nodes(), 0);
        assert_eq!(estimator.parameters(), 96);
        assert_eq!(estimator.memory_bounds(), (1552, 1559));
    }

    #[test]
    fn test_tree_allocation_size() {
        const N: usize = 2;
        let root_state = KuhnState::from_cards([0; N]);

        let estimator = TreeEstimator::<KuhnGame<N>, CfrParameter>::from_root(root_state);
        let memory_bounds = estimator.memory_bounds();

        let arena = Mutex::new(Arena::with_capacity(10000));
        let _root = allocate_tree::<KuhnGame<N>, CfrParameter>(&root_state, &arena)
            .expect("could not allocate tree");
        let size = arena.lock().unwrap().len();

        assert!(size >= memory_bounds.0, "tree is smaller than expected");
        assert!(size <= memory_bounds.1, "tree is larger than expected");
    }

    #[test]
    fn test_kuhn_solve() {
        const N: usize = 3;
        let root_state = KuhnState::from_cards([0, 1, 2]);

        let arena = {
            let estimator = TreeEstimator::<KuhnGame<N>, CfrParameter>::from_root(root_state);
            Mutex::new(Arena::with_capacity(estimator.memory_bounds().1))
        };

        let root = allocate_tree::<KuhnGame<N>, CfrParameter>(&root_state, &arena)
            .expect("could not allocate tree");

        let mut solver = Cfr::<N>;

        let mut rng = JKiss32Rng::seed_from_u64(0);

        for i in 0..100000 {
            let state = KuhnState::random(&mut rng);
            <Cfr<N> as Solver<KuhnGame<N>>>::iterate(&mut solver, root, state, i);
        }

        dump_strategy::<KuhnGame<N>, Cfr<N>, CfrParameter>(root, root_state, &solver);
    }
}
