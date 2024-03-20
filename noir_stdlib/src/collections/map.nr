use crate::cmp::Eq;
use crate::collections::vec::Vec;
use crate::option::Option;
use crate::default::Default;
use crate::hash::{Hash, Hasher, BuildHasher};
use crate::collections::bounded_vec::BoundedVec;

// We use a count of 4 buckets for each HashMap slot
// Upon exceeding it, assert will fail in order to inform the user 
// about performance degradation, so that he can adjust the capacity.
global BUCKET_ITEMS = 4;

// Hash table with open addressing and quadratic probing.
// Size of the underlying table must be known at compile time.
// It is advised to select capacity N as a power of two, or a prime number 
// because utilized probing scheme is best tailored for it.
struct HashMap<K, V, N, B> {
    _table: [Bucket<K, V>; N],

    // Amount of valid elements in the map.
    _len: u64,

    _build_hasher: B
}

// Data unit in the HashMap table.
struct Bucket<K, V> {
    items: BoundedVec<(K, V), BUCKET_ITEMS>,
}

impl<K, V> Bucket<K, V> {
    fn new() -> Self {
        Self { items: BoundedVec::new() }
    }

    fn len(self) -> u64 {
        self.items.len()
    }

    // Returns the index of the key if found.
    // Returns self.items.len() otherwise.
    fn find_slot_index(self, target_key: K) -> u64 where K: Eq {
        let mut index = self.items.len();

        for i in 0 .. BUCKET_ITEMS {
            if i < self.items.len() {
                let (key, _) = self.items.get_unchecked(i);
                if key == target_key {
                    index = i;
                }
            }
        }

        index
    }

    fn get_value(self, target_key: K) -> Option<V> where K: Eq {
        let mut result = Option::none();

        // Rewriting the find_slot_index loop here leads to one less
        // comparison since we'll already have `value`.
        for i in 0 .. BUCKET_ITEMS {
            if i < self.items.len() {
                let (key, value) = self.items.get_unchecked(i);
                if key == target_key {
                    result = Option::some(value);
                }
            }
        }

        result
    }

    // Inserts an item and returns the modified Self, along with
    // whether the item is going into a new slot or not.
    fn insert(mut self, key: K, value: V) -> (Self, bool) where K: Eq {
        let slot = self.find_slot_index(key);
        assert(slot < BUCKET_ITEMS, f"Bucket in HashMap exceeded the maximum capacity of {BUCKET_ITEMS}!");

        let new_slot = slot == self.items.len();

        if new_slot {
            self.items.push((key, value));
        } else {
            self.items.insert(slot, (key, value));
        }

        (self, new_slot)
    }

    // Removes an item and returns the modified Self, along with
    // whether an item was removed or not.
    fn remove(mut self, key: K) -> (Self, bool) where K: Eq {
        let slot = self.find_slot_index(key);

        let remove_item = slot != self.items.len();

        if remove_item {
            let _ = self.items.swap_remove(slot);
        }

        (self, remove_item)
    }

    pub fn retain(&mut self, f: fn(K, V) -> bool) {
        let mut new_items = BoundedVec::new();

        for i in 0 .. BUCKET_ITEMS {
            if i < self.items.len() {
                let (key, value) = self.items.get_unchecked(i);
                if f(key, value) {
                    new_items.push((key, value));
                }
            }
        }

        self.items = new_items;
    }
}

// While conducting lookup, we iterate attempt from 0 to N - 1 due to heuristic,
// that if we have went that far without finding desired, 
// it is very unlikely to be after - performance will be heavily degraded.
impl<K, V, N, B, H> HashMap<K, V, N, B> {
    // Creates a new instance of HashMap with specified BuildHasher.
    // docs:start:with_hasher
    pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher<H> {
        // docs:end:with_hasher
        let _table = [Bucket::new(); N];
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    // Clears the map, removing all key-value entries.
    // docs:start:clear
    pub fn clear(&mut self) {
        // docs:end:clear
        self._table = [Bucket::new(); N];
        self._len = 0;
    }

    // Returns true if the map contains a value for the specified key.
    // docs:start:contains_key
    pub fn contains_key(
        self,
        key: K
    ) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher<H>,
        H: Hasher {
        // docs:end:contains_key
        self.get(key).is_some()
    }

    // Returns true if the map contains no elements.
    // docs:start:is_empty
    pub fn is_empty(self) -> bool {
        // docs:end:is_empty
        self._len == 0
    }

    // TODO: iter functions

    // Returns a BoundedVec of all valid entries in this HashMap.
    // The length of the returned vector will always match the length of this HashMap.
    // docs:start:entries
    // pub fn entries(self) -> BoundedVec<(K, V), N> {
    //     // docs:end:entries
    //     let mut entries = BoundedVec::new();

    //     for slot in self._table {
    //         if slot.is_valid() {
    //             // SAFETY: slot.is_valid() should ensure there is a valid key-value pairing here
    //             let key_value = slot.key_value().unwrap_unchecked();
    //             entries.push(key_value);
    //         }
    //     }

    //     let msg = f"Amount of valid elements should have been {self._len} times, but got {entries.len()}.";
    //     assert(entries.len() == self._len, msg);

    //     entries
    // }

    // // Returns a BoundedVec containing all the keys within this HashMap.
    // // The length of the returned vector will always match the length of this HashMap.
    // // docs:start:keys
    // pub fn keys(self) -> BoundedVec<K, N> {
    //     // docs:end:keys
    //     let mut keys = BoundedVec::new();

    //     for slot in self._table {
    //         if slot.is_valid() {
    //             let (key, _) = slot.key_value_unchecked();
    //             keys.push(key);
    //         }
    //     }

    //     let msg = f"Amount of valid elements should have been {self._len} times, but got {keys.len()}.";
    //     assert(keys.len() == self._len, msg);

    //     keys
    // }

    // // Returns a BoundedVec containing all the values within this HashMap.
    // // The length of the returned vector will always match the length of this HashMap.
    // // docs:start:values
    // pub fn values(self) -> BoundedVec<V, N> {
    //     // docs:end:values
    //     let mut values = BoundedVec::new();

    //     for slot in self._table {
    //         if slot.is_valid() {
    //             let (_, value) = slot.key_value_unchecked();
    //             values.push(value);
    //         }
    //     }

    //     let msg = f"Amount of valid elements should have been {self._len} times, but got {values.len()}.";
    //     assert(values.len() == self._len, msg);

    //     values
    // }

    // For each key-value entry applies mutator function.
    // docs:start:iter_mut
    // pub fn iter_mut(
    //     &mut self,
    //     f: fn(K, V) -> (K, V)
    // )
    // where
    //     K: Eq + Hash,
    //     B: BuildHasher<H>,
    //     H: Hasher {
    //     // docs:end:iter_mut
    //     let mut entries = self.entries();
    //     let mut new_map = HashMap::with_hasher(self._build_hasher);

    //     for i in 0..N {
    //         if i < self._len {
    //             let entry = entries.get_unchecked(i);
    //             let (key, value) = f(entry.0, entry.1);
    //             new_map.insert(key, value);
    //         }
    //     }

    //     self._table = new_map._table;
    // }

    // // For each key applies mutator function.
    // // docs:start:iter_keys_mut
    // pub fn iter_keys_mut(
    //     &mut self,
    //     f: fn(K) -> K
    // ) 
    // where
    //     K: Eq + Hash,
    //     B: BuildHasher<H>,
    //     H: Hasher {
    //     // docs:end:iter_keys_mut
    //     let mut entries = self.entries();
    //     let mut new_map = HashMap::with_hasher(self._build_hasher);

    //     for i in 0..N {
    //         if i < self._len {
    //             let entry = entries.get_unchecked(i);
    //             let (key, value) = (f(entry.0), entry.1);
    //             new_map.insert(key, value);
    //         }
    //     }

    //     self._table = new_map._table;
    // }

    // // For each value applies mutator function.
    // // docs:start:iter_values_mut
    // pub fn iter_values_mut(&mut self, f: fn(V) -> V) {
    //     // docs:end:iter_values_mut
    //     for i in 0..N {
    //         let mut slot = self._table[i];
    //         if slot.is_valid() {
    //             let (key, value) = slot.key_value_unchecked();
    //             slot.set(key, f(value));
    //             self._table[i] = slot;
    //         }
    //     }
    // }

    // Retains only the elements specified by the predicate.
    // docs:start:retain
    pub fn retain(&mut self, f: fn(K, V) -> bool) {
        // docs:end:retain
        for index in 0..N {
            let mut slot = self._table[index];
            slot.retain(f);
            self._table[index] = slot;
        }
    }

    // Amount of active key-value entries.
    // docs:start:len
    pub fn len(self) -> u64 {
        // docs:end:len
        self._len
    }

    // Get the compile-time map capacity.
    // docs:start:capacity
    pub fn capacity(_self: Self) -> u64 {
        // docs:end:capacity
        N
    }

    // Get the value by key. If it does not exist, returns none().
    // docs:start:get
    pub fn get(
        self,
        key: K
    ) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
        // docs:end:get
        let bucket_index = self.hash(key) % N;
        let bucket = self._table[bucket_index];
        bucket.get_value(key)
    }

    // Insert key-value entry. In case key was already present, value is overridden. 
    // docs:start:insert
    pub fn insert(
        &mut self,
        key: K,
        value: V
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
        // docs:end:insert
        let bucket_index = self.hash(key) % N;
        let bucket = self._table[bucket_index];
        let (new_bucket, added) = bucket.insert(key, value);
        self._table[bucket_index] = new_bucket;

        if added {
            self._len += 1;
        }
    }

    // Removes a key-value entry. If key is not present, HashMap remains unchanged.
    // docs:start:remove
    pub fn remove(
        &mut self,
        key: K
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
        // docs:end:remove
        let bucket_index = self.hash(key) % N;
        let bucket = self._table[bucket_index];
        let (new_bucket, removed) = bucket.remove(key);
        self._table[bucket_index] = new_bucket;

        if removed {
            self._len -= 1;
        }
    }

    // Apply HashMap's hasher onto key to hash the key
    fn hash(
        self,
        key: K
    ) -> u64
    where
        K: Hash,
        B: BuildHasher<H>,
        H: Hasher {
        let mut hasher = self._build_hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as u64
    }
}

// Equality class on HashMap has to test that they have 
// equal sets of key-value entries, 
// thus one is a subset of the other and vice versa.
// docs:start:eq
impl<K, V, N, B, H> Eq for HashMap<K, V, N, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher<H>,
    H: Hasher
{
    fn eq(self, other: HashMap<K, V, N, B>) -> bool {
// docs:end:eq
        let mut equal = false;

        if self.len() == other.len() {
            equal = true;

            for bucket in self._table {
                if equal {
                    for i in 0 .. BUCKET_ITEMS {
                        if i < bucket.items.len() {
                            let (key, value) = bucket.items.get_unchecked(i);
                            let other_value = other.get(key);

                            if other_value.is_none(){
                                equal = false;
                            }else{
                                let other_value = other_value.unwrap_unchecked();
                                if value != other_value {
                                    equal = false;
                                }
                            }
                        }
                    }
                }
            }
        }

        equal
    }
}

// docs:start:default
impl<K, V, N, B, H> Default for HashMap<K, V, N, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default
{
    fn default() -> Self {
// docs:end:default
        let _build_hasher = B::default();
        let map: HashMap<K, V, N, B> = HashMap::with_hasher(_build_hasher);
        map
    }
}
