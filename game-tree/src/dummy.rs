use std::mem::{self, MaybeUninit};

use rand::Rng;

use crate::{Event, Game, Parameter, ParameterMapping, Stage};

/// Dummy everything.
#[derive(Clone)]
pub(crate) struct X;

pub(crate) type P = u8;

impl Stage for X {
    fn is_action(&self) -> bool {
        false
    }
    fn is_chance(&self) -> bool {
        false
    }
    fn is_terminal(&self) -> bool {
        false
    }
    fn player_to_act(&self) -> Option<usize> {
        None
    }
}

impl ParameterMapping for X {
    type State = X;
    fn get_parameter_count(_: &X) -> usize {
        4
    }
    fn get_parameter_index(_: &X) -> usize {
        0
    }
}

#[rustfmt::skip]
impl Game for X {
    type Action = [u8; 6];
    type Chance = [u8; 6];
    type ParameterMapping = X;
    type Stage = X;
    type State = X;

    fn advance_state(_state: &mut Self::State, _event: Event<Self::Action, Self::Chance>) { unimplemented!() }
    fn populate_events(_state: &Self::State, _events: &mut Vec<Event<Self::Action, Self::Chance>>) { unimplemented!() }
    fn get_chance_weight(_state: &Self::State, _event: Self::Chance) -> f32 { unimplemented!() }
    fn sample_chance<R: Rng>(_state: &Self::State, _rng: &mut R) -> (Self::Chance, usize) { unimplemented!() }
    fn get_stage(_state: &Self::State) -> Self::Stage { unimplemented!() }
    fn get_branching_hint(_state: &Self::State) -> usize { unimplemented!() }
    fn get_terminal_utilities(_state: &Self::State, _utilities: &mut [f32]) { unimplemented!() }
}

impl Parameter for P {
    fn initialize(parameters: &mut [MaybeUninit<Self>]) -> &mut [Self] {
        for (i, p) in parameters.iter_mut().enumerate() {
            unsafe {
                p.as_mut_ptr().write(i as P + 1);
            }
        }
        unsafe { mem::transmute(parameters) }
    }
}
