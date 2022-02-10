use std::mem::MaybeUninit;

use rand::Rng;

pub trait GameTypes {
    type Action: Copy;
    type Chance: Copy;
    type ParameterMapping: ParameterMapping<State = Self::State>;
    type Stage: Stage;
    type State;
    type Utility;
}

pub trait GameProgression: GameTypes {
    fn advance_state(state: &mut Self::State, event: Event<Self::Action, Self::Chance>);

    /// Solvers will assume that all events placed into the array are of the same Event variant.
    fn populate_events(state: &Self::State, events: &mut Vec<Event<Self::Action, Self::Chance>>);

    /// Returns the sampled chance event, and the index into the populated events array.
    fn sample_chance<R: Rng>(state: &Self::State, rng: &mut R) -> (Self::Chance, usize);

    fn get_stage(state: &Self::State) -> Self::Stage;

    /// Should return an estimate of the number of children for the node.
    /// Doesn't need to be accurate, but used by the solver to determine if full expansion of
    /// a node's children is appropriate.  For instance, a solver might want to avoid expanding
    /// very large chance nodes in a progressive tree search.
    fn get_branching_hint(state: &Self::State) -> usize;

    fn get_terminal_utilities(state: &Self::State, utilities: &mut [Self::Utility]);
}

pub trait ParameterMapping {
    type State;

    fn get_parameter_count(state: &Self::State) -> usize;
    fn get_parameter_index(state: &Self::State) -> usize;
}

pub trait Stage {
    fn is_chance(&self) -> bool;
    fn is_terminal(&self) -> bool;
}

pub trait Parameter {
    fn initialize(parameters: *mut MaybeUninit<Self>, count: usize) -> *mut Self
    where
        Self: Sized;
}

#[derive(Clone, Copy, Debug)]
pub enum Event<A, C> {
    Action(A),
    Chance(C),
}
