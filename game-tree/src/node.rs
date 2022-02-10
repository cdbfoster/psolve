use util::volatile::Volatile;

/// This will point to a node type.  The game state will know which.
pub type NodePtr = *mut ();

pub struct RootNode {
    pub first_child: Volatile<NodePtr>,
}

impl RootNode {
    /// Caller must ensure that the iterator returned does not outlive this node.
    pub fn children(&self) -> NodePtrIterator {
        NodePtrIterator::new(self.first_child.read())
    }
}

#[repr(C)]
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

#[repr(C)]
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
