use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};

use game_tree::{
    ActionNode, ChanceNode, Event, Game, NodePtr, NodePtrIterator, Parameter, ParameterMapping,
    Stage,
};
use util::volatile::Volatile;

use crate::solver::Solver;

pub struct Cfr<const N: usize>;

impl<const N: usize> Cfr<N> {
    fn cfr<'a, G>(
        &'a self,
        node: NodePtr,
        state: &G::State,
        parameter_index: usize,
        reach_probabilities: ReachProbabilities<N>,
    ) -> [f32; N]
    where
        G: 'a + Game,
    {
        let stage = G::get_stage(state);

        let mut utilities = [0.0; N];

        if stage.is_terminal() {
            G::get_terminal_utilities(state, &mut utilities);
            return utilities;
        }

        if stage.is_chance() {
            for child in node.children() {
                let chance = unsafe { (*(child.0 as *mut ChanceNode<G::Chance>)).result };

                let weight = G::get_chance_weight(state, chance);

                let mut next_state = state.clone();
                G::advance_state(&mut next_state, Event::Chance(chance));

                let next_parameter_index = self.get_parameter_index::<G>(&next_state);

                let chance_utilities = self.cfr::<G>(
                    child,
                    &next_state,
                    next_parameter_index,
                    reach_probabilities,
                );

                utilities
                    .iter_mut()
                    .zip(IntoIterator::into_iter(chance_utilities))
                    .for_each(|(u, v)| *u += v * weight);
            }
        } else {
            let (child_count, regret_sum) = {
                let mut count = 0;
                let mut sum = 0.0;

                self.parameter_iterator::<G>(node, parameter_index)
                    .map(|p| p.cumulative_regret.read())
                    .for_each(|r| {
                        count += 1;
                        sum += r.max(0.0);
                    });

                (count, sum)
            };

            let player = stage.player_to_act().unwrap();

            let mut player_action_utilities = Vec::with_capacity(child_count);

            for (child, parameters) in node
                .children()
                .zip(self.parameter_iterator::<G>(node, parameter_index))
            {
                let action =
                    unsafe { (*(child.0 as *mut ActionNode<G::Action, CfrParameter>)).action };

                // The strategic frequency of this action.
                let action_strategy = if regret_sum > 0.0 {
                    parameters.cumulative_regret.read().max(0.0) / regret_sum
                } else {
                    1.0 / child_count as f32
                };

                let next_reach = action_strategy * reach_probabilities.0[player];

                let mut next_reach_probabilities = reach_probabilities.clone();
                next_reach_probabilities.0[player] = next_reach;

                // Update cumulative strategy.
                {
                    let s = parameters.cumulative_strategy.read();
                    parameters.cumulative_strategy.write(s + next_reach);
                }

                let mut next_state = state.clone();
                G::advance_state(&mut next_state, Event::Action(action));

                let next_parameter_index = self.get_parameter_index::<G>(&next_state);

                let action_utilities = self.cfr::<G>(
                    child,
                    &next_state,
                    next_parameter_index,
                    next_reach_probabilities,
                );

                player_action_utilities.push(action_utilities[player]);

                utilities
                    .iter_mut()
                    .zip(action_utilities.iter())
                    .for_each(|(u, v)| *u += v * action_strategy);
            }

            let counterfactual_reach_probabilitiy: f32 = {
                let mut others = reach_probabilities;
                others.0[player] = 1.0;
                IntoIterator::into_iter(others.0).product()
            };

            // Update cumulative regret.
            for (parameters, utility) in self
                .parameter_iterator::<G>(node, parameter_index)
                .zip(player_action_utilities)
            {
                let regret = (utility - utilities[player]) * counterfactual_reach_probabilitiy;
                let r = parameters.cumulative_regret.read();
                parameters.cumulative_regret.write(r + regret);
            }
        }

        utilities
    }

    fn get_parameter_index<G>(&self, state: &G::State) -> usize
    where
        G: Game,
    {
        G::get_stage(&state)
            .is_action()
            .then(|| G::ParameterMapping::get_parameter_index(&state))
            .unwrap_or(0)
    }

    fn parameter_iterator<'a, G>(
        &'a self,
        action_node: NodePtr,
        parameter_index: usize,
    ) -> impl Iterator<Item = &'a CfrParameter>
    where
        G: 'a + Game,
    {
        struct ParameterIterator<'a, G> {
            children: NodePtrIterator,
            index: usize,
            _marker: PhantomData<&'a G>,
        }

        impl<'a, G> Iterator for ParameterIterator<'a, G>
        where
            G: Game,
        {
            type Item = &'a CfrParameter;

            fn next(&mut self) -> Option<Self::Item> {
                self.children
                    .next()
                    .map(|n| n.0 as *mut ActionNode<G::Action, CfrParameter>)
                    .map(|a| unsafe { &*(*a).parameters.add(self.index) })
            }
        }

        ParameterIterator::<G> {
            children: action_node.children(),
            index: parameter_index,
            _marker: PhantomData,
        }
    }
}

impl<G, const N: usize> Solver<G> for Cfr<N>
where
    G: Game,
{
    type Parameter = CfrParameter;

    fn iterate(&mut self, root: NodePtr, state: G::State, _iteration: usize) {
        self.cfr::<G>(
            root,
            &state,
            self.get_parameter_index::<G>(&state),
            ReachProbabilities([1.0; N]),
        );
    }

    fn get_utilities(&self, node: NodePtr, state: &G::State, utilities: &mut [f32])
    where
        G: Game,
    {
        unimplemented!()
    }

    fn get_strategy(
        &self,
        action_node: NodePtr,
        state: &G::State,
        alternate_index: Option<usize>,
        strategy: &mut [f32],
    ) where
        G: Game,
    {
        let parameter_index = if let Some(index) = alternate_index {
            assert!(
                index < G::ParameterMapping::get_parameter_count(state),
                "parameter index out of range"
            );
            index
        } else {
            self.get_parameter_index::<G>(state)
        };

        let (child_count, strategy_sum) = {
            let mut count = 0;
            let mut sum = 0.0;

            self.parameter_iterator::<G>(action_node, parameter_index)
                .map(|p| p.cumulative_strategy.read())
                .for_each(|p| {
                    count += 1;
                    sum += p;
                });

            (count, sum)
        };

        assert_eq!(
            strategy.len(),
            child_count,
            "incorrect size for strategy buffer"
        );

        if strategy_sum > 0.0 {
            strategy
                .iter_mut()
                .zip(
                    self.parameter_iterator::<G>(action_node, parameter_index)
                        .map(|p| p.cumulative_strategy.read()),
                )
                .for_each(|(s, t)| *s = t / strategy_sum);
        } else {
            strategy
                .iter_mut()
                .for_each(|s| *s = 1.0 / child_count as f32);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ReachProbabilities<const N: usize>([f32; N]);

#[derive(Debug)]
pub struct CfrParameter {
    pub cumulative_regret: Volatile<f32>,
    pub cumulative_strategy: Volatile<f32>,
}

impl Parameter for CfrParameter {
    fn initialize(parameters: &mut [MaybeUninit<Self>]) -> &mut [Self]
    where
        Self: Sized,
    {
        unsafe {
            parameters.as_mut_ptr().write_bytes(0, parameters.len());
            mem::transmute(parameters)
        }
    }
}
