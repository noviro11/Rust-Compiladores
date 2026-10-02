//! Tests for `Node`, `Stack`, `Queue` and `HashTable`.
//!
//! Every function has three tests:
//! - `_best`:  the normal, expected use.
//! - `_edge`:  boundary cases (empty structure, a single element, extreme values,
//!             empty keys, ...).
//! - `_worst`: misuse, failure cases and heavy load (operating on an empty structure,
//!             missing keys, every key colliding, thousands of elements, ...).
//!
//! How to use it:
//! 1. Put this file next to `main.rs`, as `src/tests.rs`.
//! 2. Add these two lines to `main.rs` (anywhere outside other blocks):
//!        #[cfg(test)]
//!        mod tests;
//! 3. Run `cargo test`.
//!
//! `#[cfg(test)]` means this module is only compiled when running `cargo test`,
//! not in the normal program. `use super::*;` imports everything from `main.rs`,
//! including private structs and functions.

use crate::hashtable::HashTable;
use crate::node::Node;
use crate::queue::Queue;
use crate::stack::Stack;

/// Number of elements used by the "heavy load" tests.
/// Kept moderate on purpose: the structs have no custom `Drop`, so a very long
/// list is freed recursively (one function call per node), and something like
/// 1,000,000 nodes could overflow the call stack when the list is dropped.
const MANY: i32 = 10_000;

// ============================================================================
// Node
// ============================================================================

#[test]
fn node_new_best() {
    let node = Node::new(5);
    assert_eq!(node.value, 5);
    assert!(node.next.is_none());
}

#[test]
fn node_new_edge() {
    // Extreme and special values are stored unchanged.
    assert_eq!(Node::new(i32::MIN).value, i32::MIN);
    assert_eq!(Node::new(i32::MAX).value, i32::MAX);
    assert_eq!(Node::new(0).value, 0);
    assert_eq!(Node::new(-1).value, -1);
}

#[test]
fn node_new_worst() {
    // Build a long chain by hand and walk it: every node must keep its own value
    // and its link to the next one.
    let mut head = Node::new(0);
    for i in 1..MANY {
        let mut new_head = Node::new(i);
        new_head.next = Some(Box::new(head));
        head = new_head;
    }

    let mut count = 1;
    let mut expected = MANY - 1;
    assert_eq!(head.value, expected);
    let mut current = head.next.as_ref();
    while let Some(node) = current {
        expected -= 1;
        assert_eq!(node.value, expected);
        count += 1;
        current = node.next.as_ref();
    }
    assert_eq!(count, MANY);
}

// ============================================================================
// Stack
// ============================================================================

// ---------- new ----------

#[test]
fn stack_new_best() {
    let s = Stack::new();
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

#[test]
fn stack_new_edge() {
    // A brand-new stack has nothing to look at or remove.
    let mut s = Stack::new();
    assert_eq!(s.peek(), None);
    assert_eq!(s.pop(), None);
}

#[test]
fn stack_new_worst() {
    // Two stacks must be completely independent.
    let mut a = Stack::new();
    let b = Stack::new();
    a.push(1);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 0);
    assert_eq!(b.peek(), None);
}

// ---------- push ----------

#[test]
fn stack_push_best() {
    let mut s = Stack::new();
    s.push(10);
    s.push(20);
    s.push(30);
    assert_eq!(s.len(), 3);
    assert_eq!(s.peek(), Some(30)); // last pushed is on top
}

#[test]
fn stack_push_edge() {
    // Extreme values and repeated values.
    let mut s = Stack::new();
    s.push(i32::MIN);
    s.push(i32::MAX);
    s.push(0);
    s.push(0);
    assert_eq!(s.len(), 4);
    assert_eq!(s.pop(), Some(0));
    assert_eq!(s.pop(), Some(0));
    assert_eq!(s.pop(), Some(i32::MAX));
    assert_eq!(s.pop(), Some(i32::MIN));
}

#[test]
fn stack_push_worst() {
    // Heavy load: many pushes.
    let mut s = Stack::new();
    for i in 0..MANY {
        s.push(i);
    }
    assert_eq!(s.len(), MANY as usize);
    assert_eq!(s.peek(), Some(MANY - 1));
}

// ---------- pop ----------

#[test]
fn stack_pop_best() {
    // LIFO: values come out in reverse order.
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    assert_eq!(s.pop(), Some(3));
    assert_eq!(s.pop(), Some(2));
    assert_eq!(s.pop(), Some(1));
    assert_eq!(s.len(), 0);
}

#[test]
fn stack_pop_edge() {
    // A single element: pop it, then the stack is empty again and still usable.
    let mut s = Stack::new();
    s.push(42);
    assert_eq!(s.pop(), Some(42));
    assert!(s.is_empty());
    assert_eq!(s.pop(), None);
    s.push(7);
    assert_eq!(s.pop(), Some(7));
}

#[test]
fn stack_pop_worst() {
    // Popping an empty stack many times must not panic, and `len` must not underflow.
    let mut s = Stack::new();
    for _ in 0..100 {
        assert_eq!(s.pop(), None);
    }
    assert_eq!(s.len(), 0);

    // Pop everything after a heavy load, in the correct order.
    for i in 0..MANY {
        s.push(i);
    }
    for i in (0..MANY).rev() {
        assert_eq!(s.pop(), Some(i));
    }
    assert_eq!(s.pop(), None);
    assert_eq!(s.len(), 0);
}

// ---------- peek ----------

#[test]
fn stack_peek_best() {
    // Peek returns the top without removing it.
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    assert_eq!(s.peek(), Some(2));
    assert_eq!(s.peek(), Some(2)); // same answer twice
    assert_eq!(s.len(), 2); // nothing was removed
}

#[test]
fn stack_peek_edge() {
    // The top changes correctly after a pop, down to a single element.
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    s.pop();
    assert_eq!(s.peek(), Some(1));
    s.pop();
    assert_eq!(s.peek(), None);
}

#[test]
fn stack_peek_worst() {
    // Peeking an empty stack returns None instead of crashing.
    let s = Stack::new();
    assert_eq!(s.peek(), None);
    assert_eq!(s.peek(), None);
    assert_eq!(s.len(), 0);
}

// ---------- is_empty ----------

#[test]
fn stack_is_empty_best() {
    let mut s = Stack::new();
    assert!(s.is_empty());
    s.push(1);
    assert!(!s.is_empty());
}

#[test]
fn stack_is_empty_edge() {
    // Becomes empty again after removing the only element.
    let mut s = Stack::new();
    s.push(1);
    s.pop();
    assert!(s.is_empty());
}

#[test]
fn stack_is_empty_worst() {
    // Many push/pop cycles, plus extra pops on an empty stack.
    let mut s = Stack::new();
    for i in 0..MANY {
        s.push(i);
        s.pop();
        s.pop(); // extra pop on an empty stack
    }
    assert!(s.is_empty());
    assert_eq!(s.len(), 0);
}

// ---------- len ----------

#[test]
fn stack_len_best() {
    let mut s = Stack::new();
    s.push(1);
    s.push(2);
    s.push(3);
    assert_eq!(s.len(), 3);
    s.pop();
    assert_eq!(s.len(), 2);
}

#[test]
fn stack_len_edge() {
    let s = Stack::new();
    assert_eq!(s.len(), 0);
}

#[test]
fn stack_len_worst() {
    // `len` stays correct under heavy load and after popping more than was pushed.
    let mut s = Stack::new();
    for i in 0..MANY {
        s.push(i);
    }
    assert_eq!(s.len(), MANY as usize);
    for _ in 0..(MANY + 10) {
        s.pop();
    }
    assert_eq!(s.len(), 0);
}

// ============================================================================
// Queue
// ============================================================================

// ---------- new ----------

#[test]
fn queue_new_best() {
    let q = Queue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

#[test]
fn queue_new_edge() {
    // A brand-new queue has nothing to look at or remove.
    let mut q = Queue::new();
    assert_eq!(q.peek(), None);
    assert_eq!(q.dequeue(), None);
}

#[test]
fn queue_new_worst() {
    // Two queues must be completely independent.
    let mut a = Queue::new();
    let b = Queue::new();
    a.enqueue(1);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 0);
    assert_eq!(b.peek(), None);
}

// ---------- enqueue ----------

#[test]
fn queue_enqueue_best() {
    let mut q = Queue::new();
    q.enqueue(10);
    q.enqueue(20);
    q.enqueue(30);
    assert_eq!(q.len(), 3);
    assert_eq!(q.peek(), Some(10)); // first in stays at the front
}

#[test]
fn queue_enqueue_edge() {
    // Extreme values and repeated values.
    let mut q = Queue::new();
    q.enqueue(i32::MIN);
    q.enqueue(i32::MAX);
    q.enqueue(0);
    q.enqueue(0);
    assert_eq!(q.dequeue(), Some(i32::MIN));
    assert_eq!(q.dequeue(), Some(i32::MAX));
    assert_eq!(q.dequeue(), Some(0));
    assert_eq!(q.dequeue(), Some(0));
}

#[test]
fn queue_enqueue_worst() {
    // Heavy load: many enqueues, front value never changes.
    let mut q = Queue::new();
    for i in 0..MANY {
        q.enqueue(i);
    }
    assert_eq!(q.len(), MANY as usize);
    assert_eq!(q.peek(), Some(0));
}

// ---------- dequeue ----------

#[test]
fn queue_dequeue_best() {
    // FIFO: values come out in the same order they went in.
    let mut q = Queue::new();
    q.enqueue(1);
    q.enqueue(2);
    q.enqueue(3);
    assert_eq!(q.dequeue(), Some(1));
    assert_eq!(q.dequeue(), Some(2));
    assert_eq!(q.dequeue(), Some(3));
    assert_eq!(q.len(), 0);
}

#[test]
fn queue_dequeue_edge() {
    // Empty the queue completely, then enqueue again.
    // This is the case where `tail` must be reset to null in `dequeue`;
    // otherwise the next enqueue writes into freed memory and the value is lost.
    let mut q = Queue::new();
    q.enqueue(1);
    assert_eq!(q.dequeue(), Some(1));
    assert!(q.is_empty());

    q.enqueue(2);
    assert_eq!(q.peek(), Some(2));
    assert_eq!(q.len(), 1);
    assert_eq!(q.dequeue(), Some(2));
    assert_eq!(q.dequeue(), None);
}

#[test]
fn queue_dequeue_worst() {
    // Dequeueing an empty queue many times must not panic, and `len` must not underflow.
    let mut q = Queue::new();
    for _ in 0..100 {
        assert_eq!(q.dequeue(), None);
    }
    assert_eq!(q.len(), 0);

    // Interleaved operations under heavy load: the queue repeatedly drains to empty
    // and refills, which exercises the tail reset many times.
    let mut next_in = 0;
    let mut next_out = 0;
    for round in 0..MANY {
        q.enqueue(next_in);
        next_in += 1;
        if round % 3 == 0 {
            q.enqueue(next_in);
            next_in += 1;
        }
        while let Some(v) = q.dequeue() {
            assert_eq!(v, next_out);
            next_out += 1;
        }
        assert!(q.is_empty());
    }
    assert_eq!(next_in, next_out);
}

// ---------- peek ----------

#[test]
fn queue_peek_best() {
    // Peek returns the front without removing it.
    let mut q = Queue::new();
    q.enqueue(1);
    q.enqueue(2);
    assert_eq!(q.peek(), Some(1));
    assert_eq!(q.peek(), Some(1)); // same answer twice
    assert_eq!(q.len(), 2); // nothing was removed
}

#[test]
fn queue_peek_edge() {
    // The front moves correctly after a dequeue, down to a single element.
    let mut q = Queue::new();
    q.enqueue(1);
    q.enqueue(2);
    q.dequeue();
    assert_eq!(q.peek(), Some(2));
    q.dequeue();
    assert_eq!(q.peek(), None);
}

#[test]
fn queue_peek_worst() {
    // Peeking an empty queue returns None instead of crashing.
    let q = Queue::new();
    assert_eq!(q.peek(), None);
    assert_eq!(q.peek(), None);
    assert_eq!(q.len(), 0);
}

// ---------- is_empty ----------

#[test]
fn queue_is_empty_best() {
    let mut q = Queue::new();
    assert!(q.is_empty());
    q.enqueue(1);
    assert!(!q.is_empty());
}

#[test]
fn queue_is_empty_edge() {
    // Becomes empty again after removing the only element.
    let mut q = Queue::new();
    q.enqueue(1);
    q.dequeue();
    assert!(q.is_empty());
}

#[test]
fn queue_is_empty_worst() {
    // Many enqueue/dequeue cycles, plus extra dequeues on an empty queue.
    let mut q = Queue::new();
    for i in 0..MANY {
        q.enqueue(i);
        q.dequeue();
        q.dequeue(); // extra dequeue on an empty queue
    }
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

// ---------- len ----------

#[test]
fn queue_len_best() {
    let mut q = Queue::new();
    q.enqueue(1);
    q.enqueue(2);
    q.enqueue(3);
    assert_eq!(q.len(), 3);
    q.dequeue();
    assert_eq!(q.len(), 2);
}

#[test]
fn queue_len_edge() {
    let q = Queue::new();
    assert_eq!(q.len(), 0);
}

#[test]
fn queue_len_worst() {
    // `len` stays correct under heavy load and after dequeuing more than was enqueued.
    let mut q = Queue::new();
    for i in 0..MANY {
        q.enqueue(i);
    }
    assert_eq!(q.len(), MANY as usize);
    for _ in 0..(MANY + 10) {
        q.dequeue();
    }
    assert_eq!(q.len(), 0);
}

// ============================================================================
// HashTable
// ============================================================================

// ---------- new ----------

#[test]
fn hashtable_new_best() {
    let t = HashTable::new(10);
    assert_eq!(t.len(), 0);
    assert_eq!(t.get("anything"), None);
}

#[test]
fn hashtable_new_edge() {
    // A table with a single bucket still works (everything goes in bucket 0).
    let mut t = HashTable::new(1);
    t.insert("a", 1);
    t.insert("b", 2);
    assert_eq!(t.get("a"), Some(1));
    assert_eq!(t.get("b"), Some(2));
}

#[test]
#[should_panic]
fn hashtable_new_worst() {
    // A table with 0 buckets can be created, but `hash` computes `hash % 0`,
    // so the first insert panics ("attempt to calculate the remainder with a divisor of zero").
    // `#[should_panic]` makes this test PASS only if the code panics.
    let mut t = HashTable::new(0);
    t.insert("a", 1);
}

// ---------- hash ----------

#[test]
fn hashtable_hash_best() {
    // The same key always gives the same index, and the index is a valid bucket.
    let t = HashTable::new(10);
    let first = t.hash("apple");
    let second = t.hash("apple");
    assert_eq!(first, second);
    assert!(first < 10);
}

#[test]
fn hashtable_hash_edge() {
    // The empty string still gives a valid index,
    // and with a single bucket every key maps to index 0.
    let t = HashTable::new(10);
    assert!(t.hash("") < 10);

    let one = HashTable::new(1);
    assert_eq!(one.hash(""), 0);
    assert_eq!(one.hash("apple"), 0);
    assert_eq!(one.hash("banana"), 0);
}

#[test]
fn hashtable_hash_worst() {
    // A very long key must not panic on overflow (the hash uses wrapping arithmetic),
    // and thousands of different keys must all land inside the bucket range.
    let t = HashTable::new(7);
    let long_key = "x".repeat(100_000);
    assert!(t.hash(&long_key) < 7);

    for i in 0..MANY {
        let key = format!("key{}", i);
        assert!(t.hash(&key) < 7);
    }
}

// ---------- insert ----------

#[test]
fn hashtable_insert_best() {
    let mut t = HashTable::new(10);
    t.insert("apple", 3);
    t.insert("banana", 5);
    assert_eq!(t.len(), 2);
    assert_eq!(t.get("apple"), Some(3));
    assert_eq!(t.get("banana"), Some(5));
}

#[test]
fn hashtable_insert_edge() {
    // Inserting an existing key updates its value instead of adding a duplicate.
    // An empty string is a valid key, and extreme values are stored unchanged.
    let mut t = HashTable::new(10);
    t.insert("apple", 3);
    t.insert("apple", 7);
    assert_eq!(t.len(), 1);
    assert_eq!(t.get("apple"), Some(7));

    t.insert("", i32::MIN);
    t.insert("max", i32::MAX);
    assert_eq!(t.get(""), Some(i32::MIN));
    assert_eq!(t.get("max"), Some(i32::MAX));
    assert_eq!(t.len(), 3);
}

#[test]
fn hashtable_insert_worst() {
    // Worst case for a hash table: every key collides in the same bucket
    // (only 1 bucket), so everything lives in one long list. It must still be correct.
    let mut t = HashTable::new(1);
    let count = 1_000;
    for i in 0..count {
        t.insert(&format!("key{}", i), i);
    }
    assert_eq!(t.len(), count as usize);
    for i in 0..count {
        assert_eq!(t.get(&format!("key{}", i)), Some(i));
    }
}

// ---------- get ----------

#[test]
fn hashtable_get_best() {
    let mut t = HashTable::new(10);
    t.insert("apple", 3);
    assert_eq!(t.get("apple"), Some(3));
    assert_eq!(t.get("apple"), Some(3)); // get does not remove anything
    assert_eq!(t.len(), 1);
}

#[test]
fn hashtable_get_edge() {
    // Keys are case-sensitive, and keys sharing a bucket are told apart.
    // ("banana", "listen" and "silent" all land in bucket 0 with 10 buckets.)
    let mut t = HashTable::new(10);
    t.insert("apple", 1);
    t.insert("listen", 2);
    t.insert("silent", 3);
    t.insert("banana", 4);
    assert_eq!(t.get("Apple"), None);
    assert_eq!(t.get("listen"), Some(2));
    assert_eq!(t.get("silent"), Some(3));
    assert_eq!(t.get("banana"), Some(4));
}

#[test]
fn hashtable_get_worst() {
    // Looking up keys that don't exist, on an empty table and on a full one.
    let mut t = HashTable::new(10);
    assert_eq!(t.get("missing"), None);

    for i in 0..MANY {
        t.insert(&format!("key{}", i), i);
    }
    assert_eq!(t.get("missing"), None);
    assert_eq!(t.get(&format!("key{}", MANY)), None); // one past the last key
}

// ---------- remove ----------

#[test]
fn hashtable_remove_best() {
    let mut t = HashTable::new(10);
    t.insert("apple", 3);
    t.insert("banana", 5);
    assert_eq!(t.remove("apple"), Some(3));
    assert_eq!(t.get("apple"), None);
    assert_eq!(t.get("banana"), Some(5)); // other keys untouched
    assert_eq!(t.len(), 1);
}

#[test]
fn hashtable_remove_edge() {
    // Removing one key from a bucket shared with others leaves the others intact,
    // and a removed key can be inserted again.
    let mut t = HashTable::new(1); // everything collides
    t.insert("a", 1);
    t.insert("b", 2);
    t.insert("c", 3);
    assert_eq!(t.remove("b"), Some(2));
    assert_eq!(t.get("a"), Some(1));
    assert_eq!(t.get("b"), None);
    assert_eq!(t.get("c"), Some(3));

    t.insert("b", 20);
    assert_eq!(t.get("b"), Some(20));
    assert_eq!(t.len(), 3);
}

#[test]
fn hashtable_remove_worst() {
    // Removing missing keys, removing the same key twice, and removing from an empty table:
    // all return None and never change `len`.
    let mut t = HashTable::new(10);
    assert_eq!(t.remove("missing"), None);
    assert_eq!(t.len(), 0);

    t.insert("apple", 3);
    assert_eq!(t.remove("apple"), Some(3));
    assert_eq!(t.remove("apple"), None);
    assert_eq!(t.len(), 0);

    // Heavy load: insert many keys, then remove them all.
    for i in 0..MANY {
        t.insert(&format!("key{}", i), i);
    }
    for i in 0..MANY {
        assert_eq!(t.remove(&format!("key{}", i)), Some(i));
    }
    assert_eq!(t.len(), 0);
}

// ---------- len ----------

#[test]
fn hashtable_len_best() {
    let mut t = HashTable::new(10);
    t.insert("a", 1);
    t.insert("b", 2);
    t.insert("c", 3);
    assert_eq!(t.len(), 3);
    t.remove("a");
    assert_eq!(t.len(), 2);
}

#[test]
fn hashtable_len_edge() {
    // An empty table has length 0, and updating an existing key doesn't change the length.
    let mut t = HashTable::new(10);
    assert_eq!(t.len(), 0);
    t.insert("a", 1);
    t.insert("a", 2);
    t.insert("a", 3);
    assert_eq!(t.len(), 1);
}

#[test]
fn hashtable_len_worst() {
    // Mixed operations: inserts, updates, removals of existing and missing keys.
    let mut t = HashTable::new(10);
    for i in 0..MANY {
        t.insert(&format!("key{}", i), i);
    }
    for i in 0..MANY {
        t.insert(&format!("key{}", i), i * 2); // updates only
    }
    assert_eq!(t.len(), MANY as usize);

    for i in 0..(MANY / 2) {
        t.remove(&format!("key{}", i));
        t.remove("missing"); // never present
    }
    assert_eq!(t.len(), (MANY / 2) as usize);
}
