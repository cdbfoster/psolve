use std::ptr;
use std::slice;
use std::sync::Mutex;

use util::arena::{Arena, Error};
use util::volatile::Volatile;

use crate::game::{Event, Game, Parameter, ParameterMapping, Stage};
use crate::node::{ActionNode, ChanceNode, NodePtr, NodePtrIterator, RootNode};

pub fn allocate_root_node(arena: &Mutex<Arena>) -> Result<NodePtr, Error> {
    let root_node = {
        let mut arena = arena.lock().unwrap();
        arena.allocate::<RootNode>(1)?
    };

    unsafe {
        (*root_node).as_mut_ptr().write(RootNode {
            _pad: ptr::null_mut(),
            first_child: Volatile::new(NodePtr::new::<()>(ptr::null_mut())),
        });
    }

    Ok(NodePtr::new(root_node))
}

/// Panics if `actions` is empty, or if `parameter_count` is 0.
pub fn allocate_action_nodes<G, P>(
    actions: &[G::Action],
    parameter_count: usize,
    arena: &Mutex<Arena>,
) -> Result<NodePtr, Error>
where
    G: Game,
    P: Parameter,
{
    assert!(
        !actions.is_empty(),
        "must provide at least one action to allocate"
    );
    assert!(
        parameter_count > 0,
        "must specify a parameter count of at least 1"
    );

    let total_parameter_count = actions.len() * parameter_count;

    let (mut current_node, parameters) = {
        let mut arena = arena.lock().unwrap();
        (
            arena.allocate::<ActionNode<G::Action, P>>(actions.len())?,
            arena.allocate::<P>(total_parameter_count)?,
        )
    };

    // Fill new nodes from the back of the memory block forward.
    unsafe {
        current_node = current_node.add(actions.len());
    }

    let mut parameters =
        P::initialize(unsafe { slice::from_raw_parts_mut(parameters, total_parameter_count) })
            .as_mut_ptr();

    // Fill new parameters from the back of the memory block forward.
    unsafe {
        parameters = parameters.add(total_parameter_count);
    }

    let mut last_node = ptr::null_mut();

    // Go in reverse to preserve the order of events in memory.
    for &action in actions.iter().rev() {
        unsafe {
            current_node = current_node.sub(1);
            parameters = parameters.sub(parameter_count);
        }

        let node = ActionNode {
            action,
            parameters,
            next_sibling: last_node,
            first_child: Volatile::new(NodePtr::new::<()>(ptr::null_mut())),
        };

        last_node = current_node as *mut ActionNode<G::Action, P>;

        unsafe {
            (*current_node).as_mut_ptr().write(node);
        }
    }

    Ok(NodePtr::new(last_node))
}

pub fn allocate_chance_nodes<G>(
    chances: &[G::Chance],
    arena: &Mutex<Arena>,
) -> Result<NodePtr, Error>
where
    G: Game,
{
    let mut current_node = {
        let mut arena = arena.lock().unwrap();
        arena.allocate::<ChanceNode<G::Chance>>(chances.len())?
    };

    // Fill new nodes from the back of the memory block forward.
    unsafe {
        current_node = current_node.add(chances.len());
    }

    let mut last_node = ptr::null_mut();

    // Go in reverse to preserve the order of events in memory.
    for &chance in chances.iter().rev() {
        unsafe {
            current_node = current_node.sub(1);
        }

        let node = ChanceNode {
            result: chance,
            next_sibling: last_node,
            first_child: Volatile::new(NodePtr::new::<()>(ptr::null_mut())),
        };

        last_node = current_node as *mut ChanceNode<G::Chance>;

        unsafe {
            (*current_node).as_mut_ptr().write(node);
        }
    }

    Ok(NodePtr::new(last_node))
}

pub fn allocate_tree<G, P>(root_state: &G::State, arena: &Mutex<Arena>) -> Result<NodePtr, Error>
where
    G: Game,
    P: Parameter,
{
    let root_node = allocate_root_node(arena)?;

    fn allocate_children<G, P>(
        state: &G::State,
        events_buffer: &mut Vec<Event<G::Action, G::Chance>>,
        arena: &Mutex<Arena>,
    ) -> Result<NodePtr, Error>
    where
        G: Game,
        P: Parameter,
    {
        let stage = G::get_stage(state);

        events_buffer.clear();
        G::populate_events(state, events_buffer);

        let first_child = if stage.is_action() {
            let actions = {
                events_buffer
                    .iter()
                    .map(|&e| match e {
                        Event::Action(a) => a,
                        _ => panic!("not an action event"),
                    })
                    .collect::<Vec<_>>()
            };

            allocate_action_nodes::<G, P>(
                &actions,
                G::ParameterMapping::get_parameter_count(state),
                arena,
            )?
        } else {
            let chances = {
                events_buffer
                    .iter()
                    .map(|e| match e {
                        Event::Chance(c) => *c,
                        _ => panic!("not a chance event"),
                    })
                    .collect::<Vec<_>>()
            };

            allocate_chance_nodes::<G>(&chances, arena)?
        };

        let mut next_events_buffer = Vec::new();

        for (&e, next_parent) in events_buffer.iter().zip(NodePtrIterator::new(first_child)) {
            let mut next_state = state.clone();
            G::advance_state(&mut next_state, e);

            if G::get_stage(&next_state).is_terminal() {
                continue;
            }

            let next_first_child =
                allocate_children::<G, P>(&next_state, &mut next_events_buffer, arena)?;

            next_parent.add_child(next_first_child);
        }

        Ok(first_child)
    }

    let mut events_buffer = Vec::new();
    let first_child = allocate_children::<G, P>(root_state, &mut events_buffer, arena)?;
    root_node.add_child(first_child);

    Ok(root_node)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::mem;

    use crate::dummy::*;
    use crate::{Game, NodePtr, ParameterMapping};

    fn assert_valid_siblings<N>(nodes: &[N]) {
        for i in 0..nodes.len() - 1 {
            if let Some(sibling) =
                NodePtr::new::<()>(unsafe { mem::transmute(&nodes[i]) }).next_sibling()
            {
                assert_eq!(
                    sibling.0 as *const N,
                    &nodes[i + 1] as *const N,
                    "sibling is incorrect"
                );
            } else {
                panic!("sibling is empty");
            }
        }
    }

    fn assert_valid_parameters<G: Game<State = X>>(nodes: &[ActionNode<G::Action, P>]) {
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
        let arena = Mutex::new(Arena::with_capacity(200));

        let events = [[1; 6], [2; 6], [3; 6]];

        let slice = unsafe {
            let ptr = allocate_action_nodes::<X, P>(
                &events,
                <X as Game>::ParameterMapping::get_parameter_count(&X),
                &arena,
            )
            .unwrap();
            std::slice::from_raw_parts(ptr.0 as *const ActionNode<[u8; 6], P>, events.len())
        };

        let buffer = arena.lock().unwrap().dump().as_ptr();

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

        let events = [[1; 6], [2; 6], [3; 6]];

        // Allocate some more.
        let slice = unsafe {
            let ptr = allocate_chance_nodes::<X>(&events, &arena).unwrap();
            std::slice::from_raw_parts_mut(ptr.0 as *mut ChanceNode<[u8; 6]>, events.len())
        };

        assert_eq!(
            buffer.wrapping_add(buffer.align_offset(mem::align_of::<ChanceNode<[u8; 6]>>())),
            slice.as_ptr() as *const u8,
        );

        assert_valid_siblings(slice);
    }
}
