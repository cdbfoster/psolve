use std::mem::MaybeUninit;

use crate::{GameTypes, Parameter, ParameterMapping, Stage};

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

impl GameTypes for X {
    type Action = [u8; 6];
    type Chance = [u8; 6];
    type ParameterMapping = X;
    type Stage = X;
    type State = X;
    type Utility = u8;
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
