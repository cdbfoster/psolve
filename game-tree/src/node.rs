use util::volatile::Volatile;

/// This will point to a node type.  The game state will know which.
pub type NodePtr = *mut ();

pub struct RootNode {
    pub first_child: Volatile<NodePtr>,
}

pub struct ActionNode<A, P> {
    pub action: A,
    pub parameters: *mut P,
    pub next_sibling: NodePtr,
    pub first_child: Volatile<NodePtr>,
}

pub struct ChanceNode<C> {
    pub result: C,
    pub next_sibling: NodePtr,
    pub first_child: Volatile<NodePtr>,
}
