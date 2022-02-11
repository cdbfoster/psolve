use rand::Rng;

use game_tree::{Event, GameProgression, GameTypes, ParameterMapping, Stage};

#[derive(Clone, Copy, Debug)]
pub enum KuhnStage {
    PlayerAction(u8),
    Showdown,
}

impl Stage for KuhnStage {
    fn is_chance(&self) -> bool {
        false
    }

    fn is_terminal(&self) -> bool {
        match self {
            KuhnStage::PlayerAction(_) => false,
            KuhnStage::Showdown => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
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
}

pub struct KuhnGameProgression<const N: usize>;

impl<const N: usize> GameTypes for KuhnGameProgression<N> {
    type Action = KuhnAction;
    type Chance = ();
    type ParameterMapping = KuhnParameterMapping<N>;
    type Stage = KuhnStage;
    type State = KuhnState<N>;
    type Utility = f32;
}

impl<const N: usize> GameProgression for KuhnGameProgression<N> {
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

    fn get_terminal_utilities(state: &Self::State, utilities: &mut [Self::Utility]) {
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
                *u = pot as f32;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use game_tree::TreeEstimator;

    #[test]
    fn test_tree_estimate() {
        const N: usize = 3;

        type CfrParameter = [f32; 2];

        let root_state = KuhnState::from_cards([0; N]);
        let estimator =
            TreeEstimator::<KuhnGameProgression<N>, CfrParameter>::from_root(root_state);

        assert_eq!(estimator.action_nodes(), 24);
        assert_eq!(estimator.chance_nodes(), 0);
        assert_eq!(estimator.parameters(), 96);
        assert_eq!(estimator.memory_bounds(), (1544, 1551));
    }
}
