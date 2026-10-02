/// Hash table: stores key-value pairs (`String` -> `i32`) and finds them quickly by key.
///
/// Uses separate chaining: there is a fixed number of buckets, and each bucket is a
/// list of `(key, value)` pairs. The hash function (djb2) turns a key into a number,
/// and `hash % number_of_buckets` decides which bucket it goes in.
/// When two keys land in the same bucket (a collision), both are stored in that bucket's list.
///
/// Fields:
/// - `buckets`: the buckets; each one is a `Vec` of `(String, i32)` pairs.
/// - `size`: the total number of pairs stored.
///
/// Functions:
/// - `HashTable::new(capacity)`: creates a table with `capacity` empty buckets.
/// - `hash(key)`: computes the bucket index for a key (used internally).
/// - `insert(key, value)`: stores the pair. If the key already exists, updates its value.
/// - `get(key)`: returns `Some(value)` if the key exists, or `None` if it doesn't.
/// - `remove(key)`: deletes the key and returns its value (`Some(value)`),
///   or `None` if it didn't exist.
/// - `len()`: the number of pairs stored.
pub struct HashTable {
    buckets: Vec<Vec<(String, i32)>>, // each bucket: a list of (key, value) pairs
    size: usize,                      // number of pairs stored
}

impl HashTable {
    pub fn new(capacity: usize) -> Self {
        HashTable {
            buckets: vec![Vec::new(); capacity], // `capacity` empty buckets
            size: 0,
        }
    }

    // djb2, turned into a bucket index
    pub fn hash(&self, key: &str) -> usize {
        let mut hash: u64 = 5381;
        for byte in key.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        (hash % self.buckets.len() as u64) as usize
    }

    pub fn insert(&mut self, key: &str, value: i32) {
        let index = self.hash(key);
        let bucket = &mut self.buckets[index];

        // key already there? update it
        for pair in bucket.iter_mut() {
            if pair.0 == key {
                pair.1 = value;
                return;
            }
        }

        // new key: add it to the bucket
        bucket.push((key.to_string(), value));
        self.size += 1;
    }

    pub fn get(&self, key: &str) -> Option<i32> {
        let index = self.hash(key);

        for pair in &self.buckets[index] {
            if pair.0 == key {
                return Some(pair.1);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &str) -> Option<i32> {
        let index = self.hash(key);
        let bucket = &mut self.buckets[index];

        for i in 0..bucket.len() {
            if bucket[i].0 == key {
                let (_, value) = bucket.remove(i); // take the pair out, keep the value
                self.size -= 1;
                return Some(value);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.size
    }
}
