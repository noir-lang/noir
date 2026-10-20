use crate::cmp::Eq;
use crate::default::Default;
use crate::hash::{BuildHasher, Hash};
use crate::option::Option;

/// A hash map for unconstrained and comptime code, mapping keys of type `K` to values of type `V`.
///
/// It is a hash table with open addressing and linear probing that grows as needed. Keys are
/// hashed with hashers built by `B`, a [`BuildHasher`]. A weak hasher only makes lookups slower;
/// it doesn't make results wrong.
///
/// All methods are `unconstrained`, except the constructors `with_hasher`,
/// `with_hasher_and_capacity` and [`Default::default`](crate::default::Default::default). Calling
/// them from constrained code needs an `unsafe` block, and their results are not constrained
/// afterward: nothing proves, for example, that a value returned by `get` was inserted under that
/// key. Check any result that constrained code relies on. Comparing two maps with `==` fails in
/// constrained code.
///
/// The standard library has no public hasher, so this example defines a minimal one, `MyHasher`.
/// The examples on this type's methods use it too. Real code should use a well-distributed hasher,
/// such as `poseidon::poseidon2::Poseidon2Hasher` from the
/// [`poseidon`](https://github.com/noir-lang/poseidon) library.
///
/// Example:
///
/// ```noir
/// use std::collections::umap::UHashMap;
/// use std::hash::{BuildHasherDefault, Hasher};
///
/// unconstrained fn main() {
///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
///     let _ = map.insert(1, 10);
///     let _ = map.insert(2, 20);
///
///     assert_eq(map.len(), 2);
///     assert_eq(map.get(1), Option::some(10));
///     assert(map.get(3).is_none());
///
///     map.remove(1);
///     assert(!map.contains_key(1));
/// }
///
/// // A minimal hasher, also used by the examples on this type's methods.
/// struct MyHasher {
///     state: Field,
/// }
///
/// impl Hasher for MyHasher {
///     fn finish(self) -> Field {
///         self.state
///     }
///
///     fn write(&mut self, input: Field) {
///         self.state = self.state * 31 + input;
///     }
/// }
///
/// impl Default for MyHasher {
///     fn default() -> Self {
///         MyHasher { state: 0 }
///     }
/// }
/// ```
pub struct UHashMap<K, V, B> {
    _table: [Slot<K, V>],

    // Amount of valid elements in the map.
    _len: u32,

    _build_hasher: B,
}

// Data unit in the UHashMap table.
// In case Noir adds support for enums in the future, this
// should be refactored to have three states:
// 1. (key, value)
// 2. (empty)
// 3. (deleted)
struct Slot<K, V> {
    _key: K,
    _value: V,
    _is_present: bool,
    _is_deleted: bool,
}

impl<K, V> Default for Slot<K, V> {
    fn default() -> Self {
        Slot {
            _key: crate::mem::zeroed(),
            _value: crate::mem::zeroed(),
            _is_present: false,
            _is_deleted: false,
        }
    }
}

impl<K, V> Slot<K, V> {
    fn is_valid(&self) -> bool {
        !self._is_deleted & self._is_present
    }

    fn set(&mut self, key: K, value: V) {
        self._key = key;
        self._value = value;
        self._is_present = true;
        self._is_deleted = false;
    }

    // Shall not override `_key_value` with Option::none(),
    // because we must be able to differentiate empty
    // and deleted slots for lookup.
    fn mark_deleted(&mut self) {
        self._is_deleted = true;
    }
}

// While conducting lookup, we iterate attempt from 0 to N - 1 due to heuristic,
// that if we have went that far without finding desired,
// it is very unlikely to be after - performance will be heavily degraded.
impl<K, V, B> UHashMap<K, V, B> {
    /// Creates a new instance of UHashMap with specified BuildHasher.
    ///
    /// The map starts empty with a one-slot table, which grows as entries are inserted.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> =
    ///         UHashMap::with_hasher(BuildHasherDefault::default());
    ///     assert(map.is_empty());
    /// }
    /// ```
    // docs:start:with_hasher
    pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher,
    {
        // docs:end:with_hasher
        let _table = [Slot::default()].as_vector();
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    /// Creates an empty map with the given BuildHasher and a table of `capacity` slots.
    ///
    /// A `capacity` of 0 is treated as 1. The table still grows as entries are inserted;
    /// starting larger avoids the early resizes.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> =
    ///         UHashMap::with_hasher_and_capacity(BuildHasherDefault::default(), 8);
    ///     assert_eq(map.capacity(), 8);
    ///     assert(map.is_empty());
    /// }
    /// ```
    pub fn with_hasher_and_capacity(_build_hasher: B, capacity: u32) -> Self
    where
        B: BuildHasher,
    {
        let capacity = if capacity == 0 { 1 } else { capacity };
        let mut _table = [].as_vector();
        for _ in 0..capacity {
            _table = _table.push_back(Slot::default());
        }
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    /// Clears the map, removing all key-value entries.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     map.clear();
    ///     assert(map.is_empty());
    /// }
    /// ```
    // docs:start:clear
    pub unconstrained fn clear(&mut self) {
        // docs:end:clear
        self._table = [Slot::default()].as_vector();
        self._len = 0;
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     assert(map.contains_key(1));
    ///     assert(!map.contains_key(2));
    /// }
    /// ```
    // docs:start:contains_key
    pub unconstrained fn contains_key(&self, key: K) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher,
    {
        // docs:end:contains_key
        self.get(key).is_some()
    }

    /// Returns true if the map contains no elements.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     assert(map.is_empty());
    ///     let _ = map.insert(1, 10);
    ///     assert(!map.is_empty());
    /// }
    /// ```
    // docs:start:is_empty
    pub unconstrained fn is_empty(&self) -> bool {
        // docs:end:is_empty
        self._len == 0
    }

    /// Returns a vector of all valid entries in this UHashMap.
    /// The length of the returned vector will always match the length of this UHashMap.
    /// The entries are in the order of the internal table, which is not the insertion order.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     let _ = map.insert(2, 20);
    ///
    ///     let entries = map.entries();
    ///     assert_eq(entries.len(), 2);
    ///     assert(entries.any(|entry| entry == (1, 10)));
    ///     assert(entries.any(|entry| entry == (2, 20)));
    /// }
    /// ```
    // docs:start:entries
    pub unconstrained fn entries(&self) -> [(K, V)] {
        // docs:end:entries
        let mut entries = [].as_vector();

        for slot in &self._table {
            if slot.is_valid() {
                // SAFETY: slot.is_valid() should ensure there is a valid key-value pairing here
                entries = entries.push_back((slot._key, slot._value));
            }
        }

        let self_len = self._len;
        let entries_len = entries.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {entries_len}.";
        assert(entries.len() == self._len, msg);

        entries
    }

    /// Returns a vector containing all the keys within this UHashMap.
    /// The length of the returned vector will always match the length of this UHashMap.
    /// The keys are in the order of the internal table, which is not the insertion order.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     let _ = map.insert(2, 20);
    ///
    ///     let keys = map.keys();
    ///     assert_eq(keys.len(), 2);
    ///     assert(keys.any(|key| key == 1));
    ///     assert(keys.any(|key| key == 2));
    /// }
    /// ```
    // docs:start:keys
    pub unconstrained fn keys(&self) -> [K] {
        // docs:end:keys
        let mut keys = [].as_vector();

        for slot in &self._table {
            if slot.is_valid() {
                keys = keys.push_back(slot._key);
            }
        }

        let self_len = self._len;
        let keys_len = keys.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {keys_len}.";
        assert(keys.len() == self._len, msg);

        keys
    }

    /// Returns a vector containing all the values within this UHashMap.
    /// The length of the returned vector will always match the length of this UHashMap.
    /// The values are in the order of the internal table, which is not the insertion order.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     let _ = map.insert(2, 20);
    ///
    ///     let values = map.values();
    ///     assert_eq(values.len(), 2);
    ///     assert(values.any(|value| value == 10));
    ///     assert(values.any(|value| value == 20));
    /// }
    /// ```
    // docs:start:values
    pub unconstrained fn values(&self) -> [V] {
        // docs:end:values
        let mut values = [].as_vector();

        for slot in &self._table {
            if slot.is_valid() {
                values = values.push_back(slot._value);
            }
        }

        let self_len = self._len;
        let values_len = values.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {values_len}.";
        assert(values.len() == self._len, msg);

        values
    }

    /// For each key-value entry applies mutator function.
    ///
    /// The map is rebuilt from the returned pairs. If `f` returns the same key for two entries,
    /// only one of them is kept.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     map.iter_mut(|key, value| (key + 100, value * 2));
    ///     assert_eq(map.get(101), Option::some(20));
    ///
    ///     // Entries mapped to the same key collapse into one.
    ///     let _ = map.insert(2, 20);
    ///     map.iter_mut(|_key, value| (0, value));
    ///     assert_eq(map.len(), 1);
    /// }
    /// ```
    // docs:start:iter_mut
    pub unconstrained fn iter_mut(&mut self, f: fn(K, V) -> (K, V))
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
        // docs:end:iter_mut
        let entries = self.entries();
        let mut new_map = UHashMap::with_hasher(self._build_hasher);

        for entry in entries {
            let (key, value) = f(entry.0, entry.1);
            let _ = new_map.insert(key, value);
        }

        self._len = new_map._len;
        self._table = new_map._table;
    }

    /// For each key applies mutator function.
    ///
    /// The map is rebuilt with the new keys. If `f` returns the same key for two entries, only one
    /// of them is kept.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     map.iter_keys_mut(|key| key + 100);
    ///     assert_eq(map.get(101), Option::some(10));
    ///     assert(!map.contains_key(1));
    ///
    ///     // Keys mapped to the same key collapse into one entry.
    ///     let _ = map.insert(2, 20);
    ///     map.iter_keys_mut(|_key| 0);
    ///     assert_eq(map.len(), 1);
    /// }
    /// ```
    // docs:start:iter_keys_mut
    pub unconstrained fn iter_keys_mut(&mut self, f: fn(K) -> K)
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
        // docs:end:iter_keys_mut
        let entries = self.entries();
        let mut new_map = UHashMap::with_hasher(self._build_hasher);

        for entry in entries {
            let (key, value) = (f(entry.0), entry.1);
            let _ = new_map.insert(key, value);
        }

        self._len = new_map._len;
        // The ownership pass being unsophisticated means we avoid a clone if we
        // set the table after the len instead of before
        self._table = new_map._table;
    }

    /// For each value applies mutator function.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     map.iter_values_mut(|value| value + 1);
    ///     assert_eq(map.get(1), Option::some(11));
    /// }
    /// ```
    // docs:start:iter_values_mut
    pub unconstrained fn iter_values_mut(&mut self, f: fn(V) -> V) {
        // docs:end:iter_values_mut
        for i in 0..self._table.len() {
            let mut slot = self._table[i];
            if slot.is_valid() {
                slot.set(slot._key, f(slot._value));
                self._table[i] = slot;
            }
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     let _ = map.insert(2, 20);
    ///     map.retain(|key, _value| key != 1);
    ///     assert_eq(map.len(), 1);
    ///     assert(!map.contains_key(1));
    /// }
    /// ```
    // docs:start:retain
    pub unconstrained fn retain(&mut self, f: fn(K, V) -> bool) {
        // docs:end:retain
        for index in 0..self._table.len() {
            let mut slot = self._table[index];
            if slot.is_valid() {
                if !f(slot._key, slot._value) {
                    slot.mark_deleted();
                    self._len -= 1;
                    self._table[index] = slot;
                }
            }
        }
    }

    /// Amount of active key-value entries.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     let _ = map.insert(1, 11);
    ///     assert_eq(map.len(), 1);
    /// }
    /// ```
    // docs:start:len
    pub unconstrained fn len(&self) -> u32 {
        // docs:end:len
        self._len
    }

    /// Get the current capacity of the inner table.
    ///
    /// Before each insertion, the table doubles if `len() + 1` is at least half the capacity, so the
    /// capacity is always more than the number of entries.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     assert(map.capacity() > map.len());
    /// }
    /// ```
    // docs:start:capacity
    pub unconstrained fn capacity(&self) -> u32 {
        // docs:end:capacity
        self._table.len()
    }

    /// Get the value by key. If it does not exist, returns none().
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     assert_eq(map.get(1), Option::some(10));
    ///     assert(map.get(2).is_none());
    /// }
    /// ```
    // docs:start:get
    pub unconstrained fn get(&self, key: K) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
        // docs:end:get
        let mut result = Option::none();

        let hash = self.hash(key);

        for attempt in 0..self._table.len() {
            let index = self.linear_probe(hash, attempt as u32);
            let slot = self._table[index];

            // Not marked as deleted and has key-value.
            if slot.is_valid() {
                if slot._key == key {
                    result = Option::some(slot._value);
                    break;
                }
            }
        }

        result
    }

    /// Insert key-value entry. In case key was already present, value is overridden.
    ///
    /// Returns true if the element was newly-inserted. In other words, this returns false
    /// if the key already has a value in this map.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     assert(map.insert(1, 10));
    ///     assert(!map.insert(1, 11));
    ///     assert_eq(map.get(1), Option::some(11));
    /// }
    /// ```
    // docs:start:insert
    pub unconstrained fn insert(&mut self, key: K, value: V) -> bool
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
        // docs:end:insert
        self.try_resize();

        let hash = self.hash(key);
        let mut insert_index: u32 = 0;
        let mut key_exists = false;
        let mut found_deleted_candidate = false;

        for attempt in 0..self._table.len() {
            let index = self.linear_probe(hash, attempt);
            let slot = self._table[index];

            if slot.is_valid() {
                if slot._key == key {
                    // Key exists: update in place without changing length.
                    insert_index = index;
                    key_exists = true;
                    break;
                }
            } else if slot._is_deleted {
                // Deleted slot: remember as candidate but keep probing for existing key.
                if !found_deleted_candidate {
                    insert_index = index;
                    found_deleted_candidate = true;
                }
            } else {
                // Empty slot: key cannot exist beyond this point.
                // Insert at a deleted slot, if we found one previously, or at the current slot.
                if !found_deleted_candidate {
                    insert_index = index;
                }
                break;
            }
        }

        let mut s = self._table[insert_index];
        s.set(key, value);
        self._table[insert_index] = s;

        // A key the probe didn't find is new wherever it lands: in the empty slot that ended the
        // probe, or in a deleted slot remembered on the way.
        if !key_exists {
            self._len += 1;
        }
        !key_exists
    }

    unconstrained fn try_resize(&mut self)
    where
        B: BuildHasher,
        K: Eq + Hash,
    {
        if self.len() + 1 >= self.capacity() / 2 {
            let capacity = self.capacity() * 2;
            let mut new_map = UHashMap::with_hasher_and_capacity(self._build_hasher, capacity);

            for entry in self.entries() {
                let _ = new_map.insert(entry.0, entry.1);
            }
            *self = new_map;
        }
    }

    /// Removes a key-value entry. If key is not present, UHashMap remains unchanged.
    ///
    /// Example:
    ///
    /// `MyHasher` is the hasher defined in the [`UHashMap`] example.
    ///
    /// ```noir
    /// use std::collections::umap::UHashMap;
    /// use std::hash::BuildHasherDefault;
    ///
    /// unconstrained fn main() {
    ///     let mut map: UHashMap<Field, Field, BuildHasherDefault<MyHasher>> = UHashMap::default();
    ///     let _ = map.insert(1, 10);
    ///     map.remove(1);
    ///     map.remove(2);
    ///     assert(map.is_empty());
    /// }
    /// ```
    // docs:start:remove
    pub unconstrained fn remove(&mut self, key: K)
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
        // docs:end:remove
        let hash = self.hash(key);

        for attempt in 0..self._table.len() {
            let index = self.linear_probe(hash, attempt as u32);
            let mut slot = self._table[index];

            // Not marked as deleted and has key-value.
            if slot.is_valid() {
                if slot._key == key {
                    slot.mark_deleted();
                    self._table[index] = slot;
                    self._len -= 1;
                    break;
                }
            }
        }
    }

    /// Apply UHashMap's hasher onto key to obtain pre-hash for probing.
    fn hash(&self, key: K) -> u32
    where
        K: Hash,
        B: BuildHasher,
    {
        let mut hasher = self._build_hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish_ref() as u32
    }

    /// Probing scheme: linear probing.
    /// Each attempt increments the index by one, wrapping around the table.
    fn linear_probe(&self, hash: u32, attempt: u32) -> u32 {
        ((hash as u64 + attempt as u64) % self._table.len() as u64) as u32
    }
}

// Equality class on UHashMap has to test that they have
// equal sets of key-value entries,
// thus one is a subset of the other and vice versa.
// docs:start:eq
impl<K, V, B> Eq for UHashMap<K, V, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher,
{
    fn eq(self, other: UHashMap<K, V, B>) -> bool {
        // docs:end:eq
        if crate::runtime::is_unconstrained() {
            // Safety: Checked above to be in unconstrained context.
            unsafe {
                let mut equal = false;

                if self.len() == other.len() {
                    equal = true;
                    for slot in self._table {
                        // Not marked as deleted and has key-value.
                        if equal & slot.is_valid() {
                            // Safety: unconstrained context
                            let other_value = other.get(slot._key);

                            if other_value.is_none() {
                                equal = false;
                            } else {
                                let other_value = other_value.unwrap_unchecked();
                                if slot._value != other_value {
                                    equal = false;
                                }
                            }
                        }
                    }
                }

                equal
            }
        } else {
            crate::panic::panic(
                "Equality check on UHashMap is not supported in constrained context",
            )
        }
    }
}

// docs:start:default
impl<K, V, B> Default for UHashMap<K, V, B>
where
    B: BuildHasher + Default,
{
    fn default() -> Self {
        // docs:end:default
        UHashMap::with_hasher(B::default())
    }
}

mod test {
    use crate::default::Default;
    use crate::hash::BuildHasherDefault;
    use crate::hash::Hasher;
    use crate::hash::poseidon2::Poseidon2Hasher;
    use crate::option::Option;
    use super::UHashMap;

    #[test]
    unconstrained fn test_general_usage() {
        let mut map = UHashMap::<Field, Field, BuildHasherDefault<Poseidon2Hasher>>::with_hasher_and_capacity(
            BuildHasherDefault::<Poseidon2Hasher> {},
            8,
        );

        assert(map.is_empty());
        assert(!map.contains_key(5));
        assert_eq(map.get(5), Option::none());

        let _ = map.insert(5, 10);
        assert(map.contains_key(5));
        assert_eq(map.get(5), Option::some(10));
        assert(!map.is_empty());

        assert_eq(map.len(), 1);

        let _ = map.insert(8, 16);

        assert_eq(map.len(), 2);

        map.iter_mut(|k, v| (k * 2, v * 2));
        assert_eq(map.get(10), Option::some(20));
        assert_eq(map.get(16), Option::some(32));

        map.iter_keys_mut(|k| k / 2);
        assert_eq(map.get(5), Option::some(20));
        assert_eq(map.get(8), Option::some(32));

        map.iter_values_mut(|v| v * 2);
        assert_eq(map.get(5), Option::some(40));
        assert_eq(map.get(8), Option::some(64));

        let mut another_map = UHashMap::<Field, Field, BuildHasherDefault<Poseidon2Hasher>>::with_hasher_and_capacity(
            BuildHasherDefault::<Poseidon2Hasher> {},
            8,
        );
        assert(map != another_map);
        let _ = another_map.insert(5, 40);
        assert(map != another_map);
        let _ = another_map.insert(7, 63);
        assert(map != another_map);
        another_map.remove(7);
        let _ = another_map.insert(8, 64);
        assert_eq(map, another_map);

        map.retain(|k, v| {
            assert((k == 5) | (k == 8));
            assert((v == 40) | (v == 64));
            k == 5
        });

        assert_eq(map.len(), 1);
        assert_eq(map.entries(), @[(5, 40)]);

        map.clear();
        assert(map.is_empty());
        assert_eq(map.len(), 0);
        assert_eq(map.values().len(), 0);
        assert_eq(map.keys().len(), 0);
        assert_eq(map.entries().len(), 0);
    }

    #[test]
    unconstrained fn test_no_duplicate_keys_after_deletion_and_insertion() {
        // The capacity and keys used here, with the Poseidon2Hasher, have the keys 5 and 8 end up in slot 0.
        let mut map = UHashMap::<Field, Field, BuildHasherDefault<Poseidon2Hasher>>::with_hasher_and_capacity(
            BuildHasherDefault::<Poseidon2Hasher> {},
            8,
        );
        assert(map.insert(5, 5));
        assert(map.insert(8, 8));
        map.remove(5);
        assert(!map.insert(8, 9));

        assert_eq(map.entries(), @[(8, 9)]);
        assert_eq(map.keys(), @[8]);
    }

    #[test]
    unconstrained fn test_with_hasher_and_capacity_zero() {
        let mut m: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>> =
            UHashMap::with_hasher_and_capacity(BuildHasherDefault::<Poseidon2Hasher> {}, 0);
        let _ = m.insert(1, 1); // This used to produce an index out of bounds
    }

    #[test]
    unconstrained fn test_length_tracks_entries_across_insert_remove_churn() {
        // `try_resize` counts live entries only, so a map that never holds more than one entry
        // never resizes, while `remove` turns every slot it empties into a deleted slot. With
        // `IdentityHasher` and capacity 8, key `i` starts its probe at slot `i % 8`, so keys 0..=7
        // leave every slot deleted and each later key lands in a deleted slot without the probe
        // ever reaching an empty one.
        let mut map = UHashMap::<Field, Field, BuildHasherDefault<IdentityHasher>>::with_hasher_and_capacity(
            BuildHasherDefault::<IdentityHasher> {},
            8,
        );

        for i in 0..16 {
            let key = i as Field;

            assert(map.insert(key, key), "inserting a fresh key reports a new entry");
            assert_eq(map.len(), 1);
            assert_eq(map.entries().len(), 1);

            map.remove(key);
            assert_eq(map.len(), 0);
            assert_eq(map.entries().len(), 0);
        }
    }

    struct IdentityHasher {
        _v: Field,
    }
    impl Default for IdentityHasher {
        fn default() -> Self {
            IdentityHasher { _v: 0 }
        }
    }
    impl Hasher for IdentityHasher {
        fn finish(self) -> Field {
            self._v
        }
        fn write(&mut self, input: Field) {
            self._v = input;
        }
    }

    struct MaxHasher {
        _v: Field,
    }
    impl Default for MaxHasher {
        fn default() -> Self {
            MaxHasher { _v: 0 }
        }
    }
    impl Hasher for MaxHasher {
        fn finish(self) -> Field {
            4294967295 // u32::MAX
        }
        fn write(&mut self, _input: Field) {}
    }

    #[test]
    unconstrained fn test_max_hash() {
        let mut m: UHashMap<Field, Field, BuildHasherDefault<MaxHasher>> = UHashMap::default();
        let _ = m.insert(1, 1);
        let _ = m.insert(2, 2); // This used to produce a math overflow
    }
}
