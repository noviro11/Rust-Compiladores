use crate::node::Node; // crate = the root of your project (lib.rs)
use std::ptr;

/// FIFO queue: the first element in is the first one out.
///
/// Implemented as a linked list of `Node`s with two ends:
/// values are added at the back (`tail`) and removed from the front (`head`).
///
/// Fields:
/// - `head`: the first node (the front of the queue). It owns the entire chain.
/// - `tail`: a raw pointer (`*mut Node`) to the last node. It owns nothing, it only
///   points. It is null (`ptr::null_mut()`) when the queue is empty.
/// - `size`: how many elements the queue holds.
///
/// Functions:
/// - `Queue::new()`: creates an empty queue.
/// - `enqueue(value)`: adds a value to the back of the queue.
/// - `dequeue()`: removes the value at the front and returns it (`Some(value)`),
///   or `None` if the queue is empty.
/// - `peek()`: returns the front value without removing it, or `None` if the queue is empty.
/// - `is_empty()`: `true` if there are no elements.
/// - `len()`: the number of elements.
pub struct Queue {
    head: Option<Box<Node>>,
    tail: *mut Node,
    size: usize,
}

impl Queue {
    pub fn new() -> Self {
        Queue {
            head: None,
            tail: ptr::null_mut(),
            size: 0,
        }
    }

    pub fn enqueue(&mut self, value: i32) {
        let mut new_node: Box<Node> = Box::new(Node::new(value)); // new node with the value we want to add
        let raw_tail: *mut Node = &mut *new_node; // we get the raw pointer of the new node
        if self.tail.is_null() {
            // empty queue: the new node is the head (and the tail, we assign the Queue tail after the if)
            self.head = Some(new_node);
        } else {
            // attach to the current last node through the raw pointer
            unsafe {
                // follows the raw pointer in tail (type ptr) and changes tail.next (ptr.next) to the new_node we created (we assign the Queue tail to this new_node pointer after the if)
                (*self.tail).next = Some(new_node);
            }
        }
        self.tail = raw_tail;
        self.size += 1;
    }

    pub fn dequeue(&mut self) -> Option<i32> {
        // we take the head node, leaving None in its place (aka Queue.head = None now)
        match self.head.take() {
            // If head is None (aka empty queue) we return nothing as there is nothing to dequeue
            None => None,
            // If we get a node in response we:
            Some(old_head) => {
                // replace the current head (None) with the .next of the old_head variable we extracted (old_head.next)
                self.head = old_head.next;
                // If the Queue head after replacing is None then:
                if self.head.is_none() {
                    // we know queue is empty we must free the queue tail
                    self.tail = ptr::null_mut();
                }
                self.size -= 1;
                // return the old_head value since dequeue
                Some(old_head.value)
            }
        }
    }

    pub fn peek(&self) -> Option<i32> {
        // we check if the head is none then:
        match self.head.as_ref() {
            // if it is None (aka the queue is empty) we return None as there is nothing to show
            None => None,
            // if there is a node in head we return the value of the result
            Some(head) => Some(head.value),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn len(&self) -> usize {
        self.size
    }
}
