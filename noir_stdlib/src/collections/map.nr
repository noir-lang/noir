use crate::cmp::Eq;
use crate::collections::bounded_vec::BoundedVec;
use crate::default::Default;
use crate::hash::{BuildHasher, Hash, Hasher};
use crate::option::Option;

// We use load factor alpha_max = 0.75.
// Upon exceeding it, assert will fail in order to inform the user
// about performance degradation, so that he can adjust the capacity.
global MAX_LOAD_FACTOR_NUMERATOR: u32 = 3;
global MAX_LOAD_FACTOR_DEN0MINATOR: u32 = 4;

/// `HashMap<Key, Value, MaxLen, Hasher>` is used to efficiently store and look up key-value pairs.
///
/// `HashMap` is a bounded type which can store anywhere from zero to `MaxLen` total elements.
/// Note that due to hash collisions, the actual maximum number of elements stored by any particular
/// hashmap is likely lower than `MaxLen`. This is true even with cryptographic hash functions since
/// every hash value will be performed modulo `MaxLen`.
///
/// Example:
///
/// ```noir
/// // Create a mapping from Fields to u32s with a maximum length of 12
/// // using a poseidon2 hasher
/// use std::hash::poseidon2::Poseidon2Hasher;
/// let mut map: HashMap<Field, u32, 12, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
///
/// map.insert(1, 2);
/// map.insert(3, 4);
///
/// let two = map.get(1).unwrap();
/// ```
pub struct HashMap<K, V, let N: u32, B> {
    _table: [Slot<K, V>; N],

    /// Amount of valid elements in the map.
    _len: u32,

    _build_hasher: B,
}

// Data unit in the HashMap table.
// In case Noir adds support for enums in the future, this
// should be refactored to have three states:
// 1. (key, value)
// 2. (empty)
// 3. (deleted)
struct Slot<K, V> {
    _key_value: Option<(K, V)>,
    _is_deleted: bool,
}

impl<K, V> Default for Slot<K, V> {
    fn default() -> Self {
        Slot { _key_value: Option::none(), _is_deleted: false }
    }
}

impl<K, V> Slot<K, V> {
    fn is_valid(self) -> bool {
        !self._is_deleted & self._key_value.is_some()
    }

    fn is_available(self) -> bool {
        self._is_deleted | self._key_value.is_none()
    }

    fn key_value(self) -> Option<(K, V)> {
        self._key_value
    }

    fn key_value_unchecked(self) -> (K, V) {
        self._key_value.unwrap_unchecked()
    }

    fn set(&mut self, key: K, value: V) {
        self._key_value = Option::some((key, value));
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
impl<K, V, let N: u32, B> HashMap<K, V, N, B> {
    /// Creates a hashmap with an existing `BuildHasher`. This can be used to ensure multiple
    /// hashmaps are created with the same hasher instance.
    ///
    /// Example:
    ///
    /// ```noir
    /// let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
    /// let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::with_hasher(my_hasher);
    /// assert(hashmap.is_empty());
    /// ```
    // docs:start:with_hasher
    pub fn with_hasher<H>(_build_hasher: B) -> Self
    where
        B: BuildHasher<H>,
    {
        // docs:end:with_hasher
        let _table = [Slot::default(); N];
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    /// Clears the hashmap, removing all key-value pairs from it.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert(!map.is_empty());
    /// map.clear();
    /// assert(map.is_empty());
    /// ```
    // docs:start:clear
    pub fn clear(&mut self) {
        // docs:end:clear
        self._table = [Slot::default(); N];
        self._len = 0;
    }

    /// Returns `true` if the hashmap contains the given key. Unlike `get`, this will not also return
    /// the value associated with the key.
    ///
    /// Example:
    ///
    /// ```noir
    /// if map.contains_key(7) {
    ///     let value = map.get(7);
    ///     assert(value.is_some());
    /// } else {
    ///     println("No value for key 7!");
    /// }
    /// ```
    // docs:start:contains_key
    pub fn contains_key<H>(self, key: K) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:contains_key
        self.get(key).is_some()
    }

    /// Returns `true` if the length of the hash map is empty.
    ///
    /// Example:
    ///
    /// ```noir
    /// assert(map.is_empty());
    ///
    /// map.insert(1, 2);
    /// assert(!map.is_empty());
    ///
    /// map.remove(1);
    /// assert(map.is_empty());
    /// ```
    // docs:start:is_empty
    pub fn is_empty(self) -> bool {
        // docs:end:is_empty
        self._len == 0
    }

    /// Returns a vector of each key-value pair present in the hashmap.
    ///
    /// The length of the returned vector is always equal to the length of the hashmap.
    ///
    /// Example:
    ///
    /// ```noir
    /// let entries = map.entries();
    ///
    /// // The length of a hashmap may not be compile-time known, so we
    /// // need to loop over its capacity instead
    /// for i in 0..map.capacity() {
    ///     if i < entries.len() {
    ///         let (key, value) = entries.get(i);
    ///         println(f"{key} -> {value}");
    ///     }
    /// }
    /// ```
    // docs:start:entries
    pub fn entries(self) -> BoundedVec<(K, V), N> {
        // docs:end:entries
        let mut entries = BoundedVec::new();

        for slot in self._table {
            if slot.is_valid() {
                // SAFETY: slot.is_valid() should ensure there is a valid key-value pairing here
                let key_value = slot.key_value().unwrap_unchecked();
                entries.push(key_value);
            }
        }

        let self_len = self._len;
        let entries_len = entries.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {entries_len}.";
        assert(entries.len() == self._len, msg);

        entries
    }

    /// Returns a vector of each key present in the hashmap.
    ///
    /// The length of the returned vector is always equal to the length of the hashmap.
    ///
    /// Example:
    ///
    /// ```noir
    /// let keys = map.keys();
    ///
    /// for i in 0..keys.max_len() {
    ///     if i < keys.len() {
    ///         let key = keys.get_unchecked(i);
    ///         let value = map.get(key).unwrap_unchecked();
    ///         println(f"{key} -> {value}");
    ///     }
    /// }
    /// ```
    // docs:start:keys
    pub fn keys(self) -> BoundedVec<K, N> {
        // docs:end:keys
        let mut keys = BoundedVec::new();

        for slot in self._table {
            if slot.is_valid() {
                let (key, _) = slot.key_value_unchecked();
                keys.push(key);
            }
        }

        let self_len = self._len;
        let keys_len = keys.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {keys_len}.";
        assert(keys.len() == self._len, msg);

        keys
    }

    /// Returns a vector of each value present in the hashmap.
    ///
    /// The length of the returned vector is always equal to the length of the hashmap.
    ///
    /// Example:
    ///
    /// ```noir
    /// let values = map.values();
    ///
    /// for i in 0..values.max_len() {
    ///     if i < values.len() {
    ///         let value = values.get_unchecked(i);
    ///         println(f"Found value {value}");
    ///     }
    /// }
    /// ```
    // docs:start:values
    pub fn values(self) -> BoundedVec<V, N> {
        // docs:end:values
        let mut values = BoundedVec::new();

        for slot in self._table {
            if slot.is_valid() {
                let (_, value) = slot.key_value_unchecked();
                values.push(value);
            }
        }

        let self_len = self._len;
        let values_len = values.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {values_len}.";
        assert(values.len() == self._len, msg);

        values
    }

    /// Iterates through each key-value pair of the HashMap, setting each key-value pair to the
    /// result returned from the given function.
    ///
    /// Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
    /// through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
    /// or `entries` if neither keys nor values need to be mutated.
    ///
    /// The iteration order is left unspecified. As a result, if two keys are mutated to become
    /// equal, which of the two values that will be present for the key in the resulting map is also unspecified.
    ///
    /// Example:
    ///
    /// ```noir
    /// // Add 1 to each key in the map, and double the value associated with that key.
    /// map.iter_mut(|k, v| (k + 1, v * 2));
    /// ```
    // docs:start:iter_mut
    pub fn iter_mut<H>(&mut self, f: fn(K, V) -> (K, V))
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:iter_mut
        let mut entries = self.entries();
        let mut new_map = HashMap::with_hasher(self._build_hasher);

        for i in 0..N {
            if i < self._len {
                let entry = entries.get_unchecked(i);
                let (key, value) = f(entry.0, entry.1);
                new_map.insert(key, value);
            }
        }

        self._table = new_map._table;
    }

    /// Iterates through the HashMap, mutating each key to the result returned from
    /// the given function.
    ///
    /// Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
    /// through. If only iteration is desired and the keys are not intended to be mutated,
    /// prefer using `entries` instead.
    ///
    /// The iteration order is left unspecified. As a result, if two keys are mutated to become
    /// equal, which of the two values that will be present for the key in the resulting map is also unspecified.
    ///
    /// Example:
    ///
    /// ```noir
    /// // Double each key, leaving the value associated with that key untouched
    /// map.iter_keys_mut(|k| k * 2);
    /// ```
    // docs:start:iter_keys_mut
    pub fn iter_keys_mut<H>(&mut self, f: fn(K) -> K)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:iter_keys_mut
        let mut entries = self.entries();
        let mut new_map = HashMap::with_hasher(self._build_hasher);

        for i in 0..N {
            if i < self._len {
                let entry = entries.get_unchecked(i);
                let (key, value) = (f(entry.0), entry.1);
                new_map.insert(key, value);
            }
        }

        self._table = new_map._table;
    }

    /// Iterates through the HashMap, applying the given function to each value and mutating the
    /// value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
    /// because the keys are untouched and the underlying hashmap thus does not need to be reordered.
    ///
    /// Example:
    ///
    /// ```noir
    /// // Halve each value
    /// map.iter_values_mut(|v| v / 2);
    /// ```
    // docs:start:iter_values_mut
    pub fn iter_values_mut(&mut self, f: fn(V) -> V) {
        // docs:end:iter_values_mut
        for i in 0..N {
            let mut slot = self._table[i];
            if slot.is_valid() {
                let (key, value) = slot.key_value_unchecked();
                slot.set(key, f(value));
                self._table[i] = slot;
            }
        }
    }

    /// Retains only the key-value pairs for which the given function returns true.
    /// Any key-value pairs for which the function returns false will be removed from the map.
    ///
    /// Example:
    ///
    /// ```noir
    /// map.retain(|k, v| (k != 0) & (v != 0));
    /// ```
    // docs:start:retain
    pub fn retain(&mut self, f: fn(K, V) -> bool) {
        // docs:end:retain
        for index in 0..N {
            let mut slot = self._table[index];
            if slot.is_valid() {
                let (key, value) = slot.key_value_unchecked();
                if !f(key, value) {
                    slot.mark_deleted();
                    self._len -= 1;
                    self._table[index] = slot;
                }
            }
        }
    }

    /// Returns the current length of this hash map.
    ///
    /// Example:
    ///
    /// ```noir
    /// // This is equivalent to checking map.is_empty()
    /// assert(map.len() == 0);
    ///
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// map.insert(5, 6);
    /// assert(map.len() == 3);
    ///
    /// // 3 was already present as a key in the hash map, so the length is unchanged
    /// map.insert(3, 7);
    /// assert(map.len() == 3);
    ///
    /// map.remove(1);
    /// assert(map.len() == 2);
    /// ```
    // docs:start:len
    pub fn len(self) -> u32 {
        // docs:end:len
        self._len
    }

    /// Returns the maximum capacity of this hashmap. This is always equal to the capacity
    /// specified in the hashmap's type.
    ///
    /// Unlike hashmaps in general purpose programming languages, hashmaps in Noir have a
    /// static capacity that does not increase as the map grows larger. Thus, this capacity
    /// is also the maximum possible element count that can be inserted into the hashmap.
    /// Due to hash collisions (modulo the hashmap length), it is likely the actual maximum
    /// element count will be lower than the full capacity.
    ///
    /// Example:
    ///
    /// ```noir
    /// let empty_map: HashMap<Field, Field, 42, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    /// assert(empty_map.len() == 0);
    /// assert(empty_map.capacity() == 42);
    /// ```
    // docs:start:capacity
    pub fn capacity(_self: Self) -> u32 {
        // docs:end:capacity
        N
    }

    /// Retrieves a value from the hashmap, returning `Option::none()` if it was not found.
    ///
    /// Example:
    ///
    /// ```noir
    /// fn get_example(map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>>) {
    ///     let x = map.get(12);
    ///
    ///     if x.is_some() {
    ///         assert(x.unwrap() == 42);
    ///     }
    /// }
    /// ```
    // docs:start:get
    pub fn get<H>(self, key: K) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:get
        let mut result = Option::none();

        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u32);
                let slot = self._table[index];

                // Not marked as deleted and has key-value.
                if slot.is_valid() {
                    let (current_key, value) = slot.key_value_unchecked();
                    if current_key == key {
                        result = Option::some(value);
                        should_break = true;
                    }
                }
            }
        }

        result
    }

    /// Inserts a new key-value pair into the map. If the key was already in the map, its
    /// previous value will be overridden with the newly provided one.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    /// map.insert(12, 42);
    /// assert(map.len() == 1);
    /// ```
    // docs:start:insert
    pub fn insert<H>(&mut self, key: K, value: V)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:insert
        self.assert_load_factor();

        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u32);
                let mut slot = self._table[index];
                let mut insert = false;

                // Either marked as deleted or has unset key-value.
                if slot.is_available() {
                    insert = true;
                    self._len += 1;
                } else {
                    let (current_key, _) = slot.key_value_unchecked();
                    if current_key == key {
                        insert = true;
                    }
                }

                if insert {
                    slot.set(key, value);
                    self._table[index] = slot;
                    should_break = true;
                }
            }
        }
    }

    /// Removes the given key-value pair from the map. If the key was not already present
    /// in the map, this does nothing.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    /// map.insert(12, 42);
    /// assert(!map.is_empty());
    ///
    /// map.remove(12);
    /// assert(map.is_empty());
    ///
    /// // If a key was not present in the map, remove does nothing
    /// map.remove(12);
    /// assert(map.is_empty());
    /// ```
    // docs:start:remove
    pub fn remove<H>(&mut self, key: K)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:remove
        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u32);
                let mut slot = self._table[index];

                // Not marked as deleted and has key-value.
                if slot.is_valid() {
                    let (current_key, _) = slot.key_value_unchecked();
                    if current_key == key {
                        slot.mark_deleted();
                        self._table[index] = slot;
                        self._len -= 1;
                        should_break = true;
                    }
                }
            }
        }
    }

    // Apply HashMap's hasher onto key to obtain pre-hash for probing.
    fn hash<H>(self, key: K) -> u32
    where
        K: Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        let mut hasher = self._build_hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as u32
    }

    // Probing scheme: quadratic function.
    // We use 0.5 constant near variadic attempt and attempt^2 monomials.
    // This ensures good uniformity of distribution for table sizes
    // equal to prime numbers or powers of two.
    fn quadratic_probe(_self: Self, hash: u32, attempt: u32) -> u32 {
        (hash + (attempt + attempt * attempt) / 2) % N
    }

    // Amount of elements in the table in relation to available slots exceeds alpha_max.
    // To avoid a comparatively more expensive division operation
    // we conduct cross-multiplication instead.
    // n / m >= MAX_LOAD_FACTOR_NUMERATOR / MAX_LOAD_FACTOR_DEN0MINATOR
    // n * MAX_LOAD_FACTOR_DEN0MINATOR >= m * MAX_LOAD_FACTOR_NUMERATOR
    fn assert_load_factor(self) {
        let lhs = self._len * MAX_LOAD_FACTOR_DEN0MINATOR;
        let rhs = self._table.len() * MAX_LOAD_FACTOR_NUMERATOR;
        let exceeded = lhs >= rhs;
        assert(!exceeded, "Load factor is exceeded, consider increasing the capacity.");
    }
}

// Equality class on HashMap has to test that they have
// equal sets of key-value entries,
// thus one is a subset of the other and vice versa.
// docs:start:eq
impl<K, V, let N: u32, B, H> Eq for HashMap<K, V, N, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher<H>,
    H: Hasher,
{
    /// Checks if two HashMaps are equal.
    ///
    /// Example:
    ///
    /// ```noir
    /// let mut map1: HashMap<Field, u64, 4, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    /// let mut map2: HashMap<Field, u64, 4, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    ///
    /// map1.insert(1, 2);
    /// map1.insert(3, 4);
    ///
    /// map2.insert(3, 4);
    /// map2.insert(1, 2);
    ///
    /// assert(map1 == map2);
    /// ```
    fn eq(self, other: HashMap<K, V, N, B>) -> bool {
        // docs:end:eq
        let mut equal = false;

        if self.len() == other.len() {
            equal = true;
            for slot in self._table {
                // Not marked as deleted and has key-value.
                if equal & slot.is_valid() {
                    let (key, value) = slot.key_value_unchecked();
                    let other_value = other.get(key);

                    if other_value.is_none() {
                        equal = false;
                    } else {
                        let other_value = other_value.unwrap_unchecked();
                        if value != other_value {
                            equal = false;
                        }
                    }
                }
            }
        }

        equal
    }
}

// docs:start:default
impl<K, V, let N: u32, B, H> Default for HashMap<K, V, N, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default,
{
    /// Constructs an empty HashMap.
    ///
    /// Example:
    ///
    /// ```noir
    /// let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    /// assert(hashmap.is_empty());
    /// ```
    fn default() -> Self {
        // docs:end:default
        let _build_hasher = B::default();
        let map: HashMap<K, V, N, B> = HashMap::with_hasher(_build_hasher);
        map
    }
}
