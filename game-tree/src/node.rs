use util::volatile::Volatile;

pub trait Node {
    fn next_sibling(&self) -> Option<*mut Self>;
}

/// This will point to a node type.  The game state will know which.
pub type NodePtr = *mut ();

#[derive(Debug)]
pub struct RootNode {
    pub first_child: Volatile<NodePtr>,
}

impl RootNode {
    /// Caller must ensure that the iterator returned does not outlive this node.
    pub fn children(&self) -> NodePtrIterator {
        NodePtrIterator::new(self.first_child.read())
    }
}

impl Node for RootNode {
    fn next_sibling(&self) -> Option<*mut Self> {
        None
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ActionNode<A, P> {
    pub next_sibling: *mut ActionNode<A, P>, // Must be first.
    pub first_child: Volatile<NodePtr>,
    pub parameters: *mut P,
    pub action: A,
}

impl<A, P> ActionNode<A, P> {
    /// Caller must ensure that the iterator returned does not outlive this node.
    pub fn children(&self) -> NodePtrIterator {
        NodePtrIterator::new(self.first_child.read())
    }
}

impl<A, P> Node for ActionNode<A, P> {
    fn next_sibling(&self) -> Option<*mut Self> {
        (!self.next_sibling.is_null()).then(|| self.next_sibling)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ChanceNode<C> {
    pub next_sibling: *mut ChanceNode<C>, // Must be first.
    pub first_child: Volatile<NodePtr>,
    pub result: C,
}

impl<C> ChanceNode<C> {
    /// Caller must ensure that the iterator returned does not outlive this node.
    pub fn children(&self) -> NodePtrIterator {
        NodePtrIterator::new(self.first_child.read())
    }
}

impl<C> Node for ChanceNode<C> {
    fn next_sibling(&self) -> Option<*mut Self> {
        (!self.next_sibling.is_null()).then(|| self.next_sibling)
    }
}

pub struct NodePtrIterator {
    next_node: NodePtr,
}

impl NodePtrIterator {
    pub fn new(node: NodePtr) -> Self {
        Self { next_node: node }
    }
}

impl Iterator for NodePtrIterator {
    type Item = NodePtr;

    fn next(&mut self) -> Option<Self::Item> {
        (!self.next_node.is_null()).then(|| {
            let ptr = self.next_node;
            self.next_node = unsafe { *(ptr as *mut NodePtr) };
            ptr
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use util::arena::{Arena, Error};

    use crate::dummy::*;
    use crate::game::{Event, GameTypes, Parameter, ParameterMapping};
    use crate::TreeAllocator;

    #[test]
    fn test_iterator() {
        let allocator = TreeAllocator::<X, P>::new(Arc::new(Mutex::new(Arena::with_capacity(300))));

        let events = [
            Event::Action([1; 6]),
            Event::Action([2; 6]),
            Event::Action([3; 6]),
            Event::Action([4; 6]),
            Event::Action([5; 6]),
        ];

        let (ptr, slice) = unsafe {
            let ptr = allocator.allocate(&X, &events).unwrap();
            (
                ptr,
                std::slice::from_raw_parts(ptr as *const ActionNode<[u8; 6], P>, events.len()),
            )
        };

        let iter = NodePtrIterator::new(ptr).collect::<Vec<_>>();

        assert_eq!(
            iter.len(),
            slice.len(),
            "incorrect number of values returned"
        );

        for (i, j) in iter
            .into_iter()
            .zip(slice.iter().map(|n| n as *const ActionNode<[u8; 6], u8>))
        {
            assert_eq!(i as *const (), j as *const (), "incorrect sibling");
        }
    }
}
