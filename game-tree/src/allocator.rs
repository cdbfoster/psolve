use std::marker::PhantomData;
use std::ptr;
use std::sync::{Arc, Mutex};

use util::arena::{Arena, Error};
use util::volatile::Volatile;

use crate::game::{Event, GameTypes, Parameter, ParameterMapping};
use crate::node::{ActionNode, ChanceNode, NodePtr};

#[derive(Clone)]
pub struct TreeAllocator<G, P> {
    arena: Arc<Mutex<Arena>>,
    _marker: PhantomData<(G, P)>,
}

impl<G, P> TreeAllocator<G, P>
where
    G: GameTypes,
    P: Parameter,
{
    pub fn new(arena: Arc<Mutex<Arena>>) -> Self {
        Self {
            arena,
            _marker: PhantomData,
        }
    }

    /// Panics if `events` is empty or if all events are not of the same Event variant.
    pub fn allocate(
        &self,
        state: &G::State,
        events: &[Event<G::Action, G::Chance>],
    ) -> Result<NodePtr, Error> {
        let node_count = events.len();
        assert!(
            node_count > 0,
            "must provide at least one event to allocate"
        );

        let node = match events[0] {
            Event::Action(_) => {
                let parameter_count = G::ParameterMapping::get_parameter_count(state);
                let total_parameter_count = node_count * parameter_count;

                let (mut current_node, parameters) = {
                    let mut arena = self.arena.lock().unwrap();
                    (
                        arena.allocate::<ActionNode<G::Action, P>>(node_count)?,
                        arena.allocate::<P>(total_parameter_count)?,
                    )
                };

                // Fill new nodes from the back of the memory block forward.
                unsafe {
                    current_node = current_node.add(node_count - 1);
                }

                let mut parameters = P::initialize(parameters, total_parameter_count);
                // Fill new parameters from the back of the memory block forward.
                unsafe {
                    parameters = parameters.add(total_parameter_count - parameter_count);
                }

                let mut last_node = ptr::null_mut();

                // Go in reverse to preserve the order of events in memory.
                for event in events.iter().rev() {
                    let action = if let Event::Action(action) = *event {
                        action
                    } else {
                        panic!("all events must be of the same type");
                    };

                    let node = ActionNode {
                        action,
                        parameters,
                        next_sibling: last_node,
                        first_child: Volatile::new(ptr::null_mut()),
                    };

                    parameters = unsafe { parameters.sub(parameter_count) };
                    last_node = current_node as *mut ActionNode<G::Action, P>;

                    unsafe {
                        (*current_node).as_mut_ptr().write(node);
                        current_node = current_node.sub(1);
                    }
                }

                last_node as NodePtr
            }
            Event::Chance(_) => {
                let mut current_node = {
                    let mut arena = self.arena.lock().unwrap();
                    arena.allocate::<ChanceNode<G::Chance>>(node_count)?
                };

                // Fill new nodes from the back of the memory block forward.
                unsafe {
                    current_node = current_node.add(node_count - 1);
                }

                let mut last_node = ptr::null_mut();

                // Go in reverse to preserve the order of events in memory.
                for event in events.iter().rev() {
                    let chance = if let Event::Chance(chance) = *event {
                        chance
                    } else {
                        panic!("all events must be of the same type");
                    };

                    let node = ChanceNode {
                        result: chance,
                        next_sibling: last_node,
                        first_child: Volatile::new(ptr::null_mut()),
                    };

                    last_node = current_node as *mut ChanceNode<G::Chance>;

                    unsafe {
                        (*current_node).as_mut_ptr().write(node);
                        current_node = current_node.sub(1);
                    }
                }

                last_node as NodePtr
            }
        };

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::mem::{self, MaybeUninit};

    use crate::{GameTypes, Node, Parameter, ParameterMapping, Stage};

    /// Dummy everything.
    struct X;

    type P = u8;

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

    fn assert_valid_siblings<N: Node>(nodes: &[N]) {
        for i in 0..nodes.len() - 1 {
            if let Some(sibling) = nodes[i].next_sibling() {
                assert_eq!(
                    sibling as *const N,
                    &nodes[i + 1] as *const N,
                    "sibling is incorrect"
                );
            } else {
                panic!("sibling is empty");
            }
        }
    }

    fn assert_valid_parameters<G: GameTypes<State = X>>(nodes: &[ActionNode<G::Action, P>]) {
        let mut parameters = {
            let ptr = nodes.last().unwrap() as *const ActionNode<G::Action, P>;
            let unaligned = ptr.wrapping_add(1) as *const P;
            unaligned.wrapping_add(unaligned.align_offset(mem::align_of::<P>())) as *const P
        };

        let parameter_count = G::ParameterMapping::get_parameter_count(&X);

        for (i, node) in nodes.iter().enumerate() {
            assert_eq!(
                node.parameters as *const P, parameters,
                "parameters are incorrect"
            );
            assert_eq!(
                &(i as P * parameter_count as P + 1
                    ..i as P * parameter_count as P + 1 + parameter_count as P)
                    .collect::<Vec<_>>(),
                unsafe { std::slice::from_raw_parts(node.parameters, parameter_count) },
                "parameters have incorrect values",
            );
            parameters = parameters.wrapping_add(parameter_count);
        }
    }

    #[test]
    fn test_allocate() {
        let arena = Arena::with_capacity(200);
        let allocator = TreeAllocator::<X, P>::new(Arc::new(Mutex::new(arena)));

        let events = [
            Event::Action([1; 6]),
            Event::Action([2; 6]),
            Event::Action([3; 6]),
        ];

        let slice = unsafe {
            let ptr = allocator.allocate(&X, &events).unwrap();
            std::slice::from_raw_parts(ptr as *const ActionNode<[u8; 6], P>, events.len())
        };

        let buffer = allocator.arena.lock().unwrap().dump().as_ptr();

        // The location of the returned nodes should be the allocator's buffer + alignment offset of a node.
        assert_eq!(
            buffer.wrapping_add(buffer.align_offset(mem::align_of::<ActionNode<[u8; 6], P>>())),
            slice.as_ptr() as *const u8,
        );

        assert_valid_siblings(slice);
        assert_valid_parameters::<X>(slice);

        // Advance to the end of the allocated block.
        let buffer = {
            let ptr = buffer
                .wrapping_add(buffer.align_offset(mem::align_of::<ActionNode<[u8; 6], P>>()))
                .wrapping_add(events.len() * mem::size_of::<ActionNode<[u8; 6], P>>());
            ptr.wrapping_add(ptr.align_offset(mem::align_of::<P>()))
                .wrapping_add(X::get_parameter_count(&X) * events.len() * mem::size_of::<P>())
        };

        let events = [
            Event::Chance([1; 6]),
            Event::Chance([2; 6]),
            Event::Chance([3; 6]),
        ];

        // Allocate some more.
        let slice = unsafe {
            let ptr = allocator.allocate(&X, &events).unwrap();
            std::slice::from_raw_parts(ptr as *const ChanceNode<[u8; 6]>, events.len())
        };

        assert_eq!(
            buffer.wrapping_add(buffer.align_offset(mem::align_of::<ChanceNode<[u8; 6]>>())),
            slice.as_ptr() as *const u8,
        );

        assert_valid_siblings(slice);
    }
}
