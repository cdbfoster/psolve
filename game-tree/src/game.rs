use std::fmt;
use std::mem::MaybeUninit;

use rand::Rng;

pub trait Game {
    type Action: Copy + fmt::Debug;
    type Chance: Copy + fmt::Debug;
    type ParameterMapping: ParameterMapping<State = Self::State>;
    type Stage: Stage;
    type State: Clone;

    fn advance_state(state: &mut Self::State, event: Event<Self::Action, Self::Chance>);

    /// Solvers will assume that all events placed into the array are of the same Event variant.
    fn populate_events(state: &Self::State, events: &mut Vec<Event<Self::Action, Self::Chance>>);

    fn get_chance_weight(state: &Self::State, event: Self::Chance) -> f32;

    /// Returns the sampled chance event, and the index into the populated events array.
    fn sample_chance<R: Rng>(state: &Self::State, rng: &mut R) -> (Self::Chance, usize);

    fn get_stage(state: &Self::State) -> Self::Stage;

    /// Should return an estimate of the number of children for the node.
    /// Doesn't need to be accurate, but used by the solver to determine if full expansion of
    /// a node's children is appropriate.  For instance, a solver might want to avoid expanding
    /// very large chance nodes in a progressive tree search.
    fn get_branching_hint(state: &Self::State) -> usize;

    fn get_terminal_utilities(state: &Self::State, utilities: &mut [f32]);
}

pub trait ParameterMapping {
    type State;

    fn get_parameter_count(state: &Self::State) -> usize;
    fn get_parameter_index(state: &Self::State) -> usize;
}

pub trait Stage {
    fn is_action(&self) -> bool;
    fn is_chance(&self) -> bool;
    fn is_terminal(&self) -> bool;

    fn player_to_act(&self) -> Option<usize>;
}

pub trait Parameter {
    fn initialize(parameters: &mut [MaybeUninit<Self>]) -> &mut [Self]
    where
        Self: Sized;
}

#[derive(Clone, Copy)]
pub enum Event<A, C> {
    Action(A),
    Chance(C),
}

impl<A, C> fmt::Debug for Event<A, C>
where
    A: fmt::Debug,
    C: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Action(a) => a.fmt(f),
            Event::Chance(c) => c.fmt(f),
        }
    }
}
