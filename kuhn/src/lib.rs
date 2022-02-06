use rand::Rng;

use game_tree::{Event, GameProgression, ParameterMap, Stage};

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

pub struct KuhnState<const N: usize> {
    cards: [u8; N],
    bettor: Option<u8>,
    called: [bool; N],
    stage: KuhnStage,
}

pub struct KuhnGameProgression<const N: usize>;

impl<const N: usize> GameProgression for KuhnGameProgression<N> {
    type Stage = KuhnStage;
    type Action = KuhnAction;
    type Chance = ();
    type State = KuhnState<N>;

    fn advance_state(state: &mut Self::State, event: Event<Self::Action, Self::Chance>) {
        if let Event::Action(action) = event {
            if let KuhnStage::PlayerAction(player) = state.stage {
                if let Some(bettor) = state.bettor {
                    if matches!(action, KuhnAction::Bet) {
                        // This is a call.
                        state.called[player as usize] = true;
                    }

                    let next_player = (player + 1) as usize % N;
                    if state.called[next_player] || next_player == bettor as usize {
                        state.stage = KuhnStage::Showdown;
                    } else {
                        state.stage = KuhnStage::PlayerAction(next_player as u8);
                    }
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
}

pub struct KuhnParameterMap<const N: usize>;

impl<const N: usize> ParameterMap for KuhnParameterMap<N> {
    type State = KuhnState<N>;
    type Parameter = [f32; 2];

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
