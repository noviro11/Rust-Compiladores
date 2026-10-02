use crate::node::Node; // crate = the root of your project (lib.rs)

/// LIFO stack: the last element in is the first one out.
///
/// Implemented as a linked list of `Node`s. `head` is the top of the stack,
/// and every operation happens there, so all operations are O(1).
/// It has no maximum capacity: it grows one node at a time.
///
/// Fields:
/// - `head`: the top node, or `None` if the stack is empty.
/// - `size`: how many elements the stack holds.
///
/// Functions:
/// - `Stack::new()`: creates an empty stack.
/// - `push(value)`: adds a value to the top.
/// - `pop()`: removes the top value and returns it (`Some(value)`),
///   or `None` if the stack is empty.
/// - `peek()`: returns the top value without removing it, or `None` if the stack is empty.
/// - `is_empty()`: `true` if there are no elements.
/// - `len()`: the number of elements.
pub struct Stack {
    head: Option<Box<Node>>,
    size: usize,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            head: None,
            size: 0,
        }
    }

    pub fn push(&mut self, value: i32) {
        let mut new_node = Node::new(value); // create the node (next is None by default)
        new_node.next = self.head.take(); // new node points to the old head; head is now None (because of take)
        self.head = Some(Box::new(new_node)); // wrap the node in Box<> and make it the new head
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.head.take() {
            None => None, // if stack empty return None for pop
            Some(old_head) => {
                // if old_head exists (aka the stack current head)
                self.head = old_head.next; // assign the next node as the new head
                self.size -= 1;
                Some(old_head.value) // return value of old_head (no return needed somehow)
            }
        }
    }

    pub fn peek(&self) -> Option<i32> {
        // we check if the head is none then:
        match self.head.as_ref() {
            // if it is the stack is empty and we return none
            None => None,
            // if it is not we return the value of the head node
            Some(node) => Some(node.value),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn len(&self) -> usize {
        self.size
    }
}
