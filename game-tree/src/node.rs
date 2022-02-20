use util::volatile::Volatile;

/// This will point to a node type.  The game state will know which.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct NodePtr(pub *mut ());

impl NodePtr {
    pub fn new<T>(ptr: *mut T) -> Self {
        Self(ptr as *mut ())
    }

    /// Caller must ensure that the iterator returned does not outlive this node.
    pub fn children(&self) -> NodePtrIterator {
        let node = self.0 as *mut NodeRelationships;
        NodePtrIterator::new(unsafe { (*node).first_child.read() })
    }

    pub fn next_sibling(&self) -> Option<NodePtr> {
        let node = self.0 as *mut NodeRelationships;
        let next_sibling = unsafe { (*node).next_sibling };
        (!next_sibling.0.is_null()).then(|| next_sibling)
    }

    /// Caller must ensure that `child` is the same type as any other children.
    pub fn add_child(&self, child: NodePtr) {
        let node = self.0 as *mut NodeRelationships;
        let sibling = unsafe { (*node).first_child.read() };
        if !sibling.0.is_null() {
            unsafe {
                (*(child.0 as *mut NodeRelationships)).next_sibling = sibling;
            }
        }
        unsafe {
            (*node).first_child.write(child);
        }
    }
}

/// Must match the layout of the beginning of all node types.
#[repr(C)]
pub struct NodeRelationships {
    pub next_sibling: NodePtr,
    pub first_child: Volatile<NodePtr>,
}

#[repr(C)]
#[derive(Debug)]
pub struct RootNode {
    pub _pad: *mut (), // Must be first.
    pub first_child: Volatile<NodePtr>,
}

#[repr(C)]
#[derive(Debug)]
pub struct ActionNode<A, P> {
    pub next_sibling: *mut ActionNode<A, P>,
    pub first_child: Volatile<NodePtr>,
    pub parameters: *mut P,
    pub action: A,
}

#[repr(C)]
#[derive(Debug)]
pub struct ChanceNode<C> {
    pub next_sibling: *mut ChanceNode<C>,
    pub first_child: Volatile<NodePtr>,
    pub result: C,
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
        (!self.next_node.0.is_null()).then(|| {
            let ptr = self.next_node;
            self.next_node = unsafe { (*(ptr.0 as *mut NodeRelationships)).next_sibling };
            ptr
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use util::arena::Arena;

    use crate::allocate_action_nodes;
    use crate::dummy::*;
    use crate::game::{Game, ParameterMapping};

    #[test]
    fn test_iterator() {
        let arena = Mutex::new(Arena::with_capacity(300));

        let events = [[1; 6], [2; 6], [3; 6], [4; 6], [5; 6]];

        let (ptr, slice) = unsafe {
            let ptr = allocate_action_nodes::<X, P>(
                &events,
                <X as Game>::ParameterMapping::get_parameter_count(&X),
                &arena,
            )
            .unwrap();
            (
                ptr,
                std::slice::from_raw_parts(ptr.0 as *const ActionNode<[u8; 6], P>, events.len()),
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
            assert_eq!(i.0 as *const (), j as *const (), "incorrect sibling");
        }
    }
}
