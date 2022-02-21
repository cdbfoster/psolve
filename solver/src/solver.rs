use std::fmt::Debug;

use game_tree::{ActionNode, Event, Game, NodePtr, ParameterMapping, Stage};

pub trait Solver<G>
where
    G: Game,
{
    type Parameter;

    fn iterate(&mut self, root: NodePtr, state: G::State, iteration: usize);

    fn get_utilities(&self, node: NodePtr, state: &G::State, utilities: &mut [f32])
    where
        G: Game;

    /// Caller must make sure `action_node` is really an ActionNode.
    /// If `alternate_index` is specified, return the strategy for the parameter at that
    /// index instead of the one indicated by the state.
    fn get_strategy(
        &self,
        action_node: NodePtr,
        state: &G::State,
        alternate_index: Option<usize>,
        strategy: &mut [f32],
    ) where
        G: Game;
}

pub fn dump_strategy<G, S, P>(root: NodePtr, state: G::State, solver: &S)
where
    G: Game,
    S: Solver<G, Parameter = P>,
    P: Debug,
{
    /// An action, followed by the frequency of that action, and optionally the parameter
    /// associated with this information state.  The parameter might not be present if
    /// some nodes don't have all of their actions expanded.
    type StrategyAction<'a, A, P> = (A, (f32, Option<&'a P>));

    fn print_information_state<G, P>(
        hidden_information: &str,
        history: &[Event<G::Action, G::Chance>],
        strategy: &[StrategyAction<'_, G::Action, P>],
    ) where
        G: Game,
        P: Debug,
    {
        println!("Hidden Information: {}", hidden_information);
        println!("History: {:?}", history);
        println!("Strategy: [");
        for (action, (frequency, parameter)) in strategy {
            println!(
                "  {:<20} {:6.2}%  {}",
                format!("{:?}:", action),
                frequency * 100.0,
                if let Some(parameter) = parameter {
                    format!("{:?}", parameter)
                } else {
                    String::from("(not present in tree)")
                },
            )
        }
        println!("]\n");
    }

    fn descend<G, S, P>(
        node: NodePtr,
        state: G::State,
        solver: &S,
        history: Vec<Event<G::Action, G::Chance>>,
    ) where
        G: Game,
        S: Solver<G, Parameter = P>,
        P: Debug,
    {
        let stage = G::get_stage(&state);

        if stage.is_action() {
            let mut actions = Vec::with_capacity(G::get_branching_hint(&state));
            G::populate_events(&state, &mut actions);

            let mut strategy = vec![0.0; actions.len()];

            for i in 0..G::ParameterMapping::get_parameter_count(&state) {
                solver.get_strategy(node, &state, Some(i), &mut strategy);

                let description = G::ParameterMapping::get_parameter_description(&state, Some(i));

                let mut bundled_strategy = Vec::with_capacity(actions.len());
                for a in actions
                    .iter()
                    .map(|&e| match e {
                        Event::Action(a) => a,
                        _ => panic!("event must be action"),
                    })
                    .enumerate()
                    .map(|(j, a)| {
                        (
                            a,
                            (
                                strategy[j],
                                node.children()
                                    .map(|n| n.0 as *mut ActionNode<G::Action, P>)
                                    .find(|&b| unsafe { (*b).action } == a)
                                    .map(|b| unsafe { &*(*b).parameters.add(i) }),
                            ),
                        )
                    })
                {
                    bundled_strategy.push(a);
                }

                print_information_state::<G, P>(&description, &history, &bundled_strategy);
            }

            for child in node.children() {
                let action = unsafe { (*(child.0 as *mut ActionNode<G::Action, P>)).action };

                let event = Event::Action(action);

                let mut next_state = state.clone();
                G::advance_state(&mut next_state, event);

                let mut next_history = history.clone();
                next_history.push(event);

                descend::<G, S, P>(child, next_state, solver, next_history);
            }
        }
    }

    descend::<G, S, P>(root, state, solver, Vec::new())
}
