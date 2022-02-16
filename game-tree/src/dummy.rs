use std::mem::MaybeUninit;

use rand::Rng;

use crate::{Event, Game, Parameter, ParameterMapping, Stage};

/// Dummy everything.
pub(crate) struct X;

pub(crate) type P = u8;

impl Stage for X {
    fn is_chance(&self) -> bool {
        false
    }
    fn is_terminal(&self) -> bool {
        false
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
    type Utility = u8;

    fn advance_state(_state: &mut Self::State, _event: Event<Self::Action, Self::Chance>) { unimplemented!() }
    fn populate_events(_state: &Self::State, _events: &mut Vec<Event<Self::Action, Self::Chance>>) { unimplemented!() }
    fn sample_chance<R: Rng>(_state: &Self::State, _rng: &mut R) -> (Self::Chance, usize) { unimplemented!() }
    fn get_stage(_state: &Self::State) -> Self::Stage { unimplemented!() }
    fn get_branching_hint(_state: &Self::State) -> usize { unimplemented!() }
    fn get_terminal_utilities(_state: &Self::State, _utilities: &mut [Self::Utility]) { unimplemented!() }
}

impl Parameter for P {
    fn initialize(parameters: *mut MaybeUninit<Self>, count: usize) -> *mut Self {
        let mut cur = parameters;
        for i in 1..count as P + 1 {
            unsafe {
                (*cur).as_mut_ptr().write(i);
                cur = cur.add(1);
            }
        }
        parameters as *mut P
    }
}
