use std::marker::PhantomData;
use std::mem;

use util::arena::DummyArena;

use crate::game::{Event, Game, ParameterMapping, Stage};
use crate::node::{ActionNode, ChanceNode, RootNode};

pub struct TreeEstimator<G, P> {
    action: usize,
    chance: usize,
    parameters: usize,
    arena: DummyArena,
    _marker: PhantomData<(G, P)>,
}

impl<G, P> TreeEstimator<G, P>
where
    G: Game,
    G::State: Clone,
{
    pub fn from_root(root_state: G::State) -> Self {
        let mut estimator = Self {
            action: 0,
            chance: 0,
            parameters: 0,
            arena: DummyArena::infinite(),
            _marker: PhantomData,
        };

        estimator.arena.allocate::<RootNode>(1).unwrap();

        let mut events_buffer = Vec::new();
        estimator.build_tree(root_state, &mut events_buffer);

        estimator
    }

    pub fn action_nodes(&self) -> usize {
        self.action
    }

    pub fn chance_nodes(&self) -> usize {
        self.chance
    }

    pub fn parameters(&self) -> usize {
        self.parameters
    }

    /// Returns the minimum and maximum number of bytes the tree will occupy in memory.
    /// The maximum is the exact amount + the worst-case alignment offset.
    pub fn memory_bounds(&self) -> (usize, usize) {
        let max_offset = mem::align_of::<RootNode>() - 1;
        (self.arena.len(), self.arena.len() + max_offset)
    }

    fn build_tree(
        &mut self,
        state: G::State,
        events_buffer: &mut Vec<Event<G::Action, G::Chance>>,
    ) {
        let stage = G::get_stage(&state);

        if stage.is_terminal() {
            return;
        }

        events_buffer.clear();
        G::populate_events(&state, events_buffer);

        if stage.is_action() {
            self.arena
                .allocate::<ActionNode<G::Action, P>>(events_buffer.len())
                .unwrap();
            self.action += events_buffer.len();

            let parameters = events_buffer.len() * G::ParameterMapping::get_parameter_count(&state);
            self.arena.allocate::<P>(parameters).unwrap();
            self.parameters += parameters;
        } else {
            self.arena
                .allocate::<ChanceNode<G::Chance>>(events_buffer.len())
                .unwrap();
            self.chance += events_buffer.len();
        }

        let mut next_events_buffer = Vec::new();

        for &mut e in events_buffer {
            let mut next_state = state.clone();
            G::advance_state(&mut next_state, e);
            self.build_tree(next_state, &mut next_events_buffer);
        }
    }
}
