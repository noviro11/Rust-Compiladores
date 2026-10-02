/// A node in a singly linked list. Used by `Stack` and `Queue`.
///
/// Each node holds a value and owns the next node in the list.
///
/// Fields:
/// - `value`: the number stored in the node.
/// - `next`: the next node. `Some(Box<Node>)` if there is one, `None` if this is the last node.
///   The `Box` stores the node on the heap and is its only owner (like a `unique_ptr` in C++),
///   so freeing a node also frees the rest of the chain after it.
///
/// Functions:
/// - `Node::new(value)`: creates a node with that value and no next node (`next = None`).
pub struct Node {
    pub value: i32,
    pub next: Option<Box<Node>>,
}

impl Node {
    pub fn new(value: i32) -> Self {
        Node {
            value: value,
            next: None,
        }
    }
}
