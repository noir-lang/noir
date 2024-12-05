use crate::cmp::Eq;
use crate::default::Default;
use crate::hash::{BuildHasher, Hash, Hasher};
use crate::option::Option;

// An unconstrained hash table with open addressing and quadratic probing.
// Note that "unconstrained" here means that almost all operations on this
// map are unconstrained and importantly are not constrained afterward either.
// This map is meant to be used in unconstrained or comptime code where this
// is not an issue.
//
// Compared to the constrained HashMap type, UHashMap can grow automatically
// as needed and is more efficient since it can break out of loops early.
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
impl<K, V, B> UHashMap<K, V, B> {
    // Creates a new instance of UHashMap with specified BuildHasher.
    // docs:start:with_hasher
    pub fn with_hasher<H>(_build_hasher: B) -> Self
    where
        B: BuildHasher<H>,
    {
        // docs:end:with_hasher
        let _table = &[Slot::default()];
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    pub fn with_hasher_and_capacity<H>(_build_hasher: B, capacity: u32) -> Self
    where
        B: BuildHasher<H>,
    {
        // docs:end:with_hasher
        let mut _table = &[];
        for _ in 0..capacity {
            _table = _table.push_back(Slot::default());
        }
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    // Clears the map, removing all key-value entries.
    // docs:start:clear
    pub fn clear(&mut self) {
        // docs:end:clear
        self._table = &[Slot::default()];
        self._len = 0;
    }

    // Returns true if the map contains a value for the specified key.
    // docs:start:contains_key
    pub fn contains_key<H>(self, key: K) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:contains_key
        unsafe { self.get(key) }.is_some()
    }

    // Returns true if the map contains no elements.
    // docs:start:is_empty
    pub fn is_empty(self) -> bool {
        // docs:end:is_empty
        self._len == 0
    }

    // Returns a BoundedVec of all valid entries in this UHashMap.
    // The length of the returned vector will always match the length of this UHashMap.
    // docs:start:entries
    pub fn entries(self) -> [(K, V)] {
        // docs:end:entries
        let mut entries = &[];

        for slot in self._table {
            if slot.is_valid() {
                // SAFETY: slot.is_valid() should ensure there is a valid key-value pairing here
                let key_value = slot.key_value().unwrap_unchecked();
                entries = entries.push_back(key_value);
            }
        }

        let self_len = self._len;
        let entries_len = entries.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {entries_len}.";
        assert(entries.len() == self._len, msg);

        entries
    }

    // Returns a BoundedVec containing all the keys within this UHashMap.
    // The length of the returned vector will always match the length of this UHashMap.
    // docs:start:keys
    pub fn keys(self) -> [K] {
        // docs:end:keys
        let mut keys = &[];

        for slot in self._table {
            if slot.is_valid() {
                let (key, _) = slot.key_value_unchecked();
                keys = keys.push_back(key);
            }
        }

        let self_len = self._len;
        let keys_len = keys.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {keys_len}.";
        assert(keys.len() == self._len, msg);

        keys
    }

    // Returns a BoundedVec containing all the values within this UHashMap.
    // The length of the returned vector will always match the length of this UHashMap.
    // docs:start:values
    pub fn values(self) -> [V] {
        // docs:end:values
        let mut values = &[];

        for slot in self._table {
            if slot.is_valid() {
                let (_, value) = slot.key_value_unchecked();
                values = values.push_back(value);
            }
        }

        let self_len = self._len;
        let values_len = values.len();
        let msg =
            f"Amount of valid elements should have been {self_len} times, but got {values_len}.";
        assert(values.len() == self._len, msg);

        values
    }

    // For each key-value entry applies mutator function.
    // docs:start:iter_mut
    pub unconstrained fn iter_mut<H>(&mut self, f: fn(K, V) -> (K, V))
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:iter_mut
        let mut entries = self.entries();
        let mut new_map = UHashMap::with_hasher(self._build_hasher);

        for entry in entries {
            let (key, value) = f(entry.0, entry.1);
            new_map.insert(key, value);
        }

        self._table = new_map._table;
    }

    // For each key applies mutator function.
    // docs:start:iter_keys_mut
    pub unconstrained fn iter_keys_mut<H>(&mut self, f: fn(K) -> K)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:iter_keys_mut
        let mut entries = self.entries();
        let mut new_map = UHashMap::with_hasher(self._build_hasher);

        for entry in entries {
            let (key, value) = (f(entry.0), entry.1);
            new_map.insert(key, value);
        }

        self._table = new_map._table;
    }

    // For each value applies mutator function.
    // docs:start:iter_values_mut
    pub fn iter_values_mut(&mut self, f: fn(V) -> V) {
        // docs:end:iter_values_mut
        for i in 0..self._table.len() {
            let mut slot = self._table[i];
            if slot.is_valid() {
                let (key, value) = slot.key_value_unchecked();
                slot.set(key, f(value));
                self._table[i] = slot;
            }
        }
    }

    // Retains only the elements specified by the predicate.
    // docs:start:retain
    pub fn retain(&mut self, f: fn(K, V) -> bool) {
        // docs:end:retain
        for index in 0..self._table.len() {
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

    // Amount of active key-value entries.
    // docs:start:len
    pub fn len(self) -> u32 {
        // docs:end:len
        self._len
    }

    // Get the current capacity of the inner table.
    // docs:start:capacity
    pub fn capacity(self: Self) -> u32 {
        // docs:end:capacity
        self._table.len()
    }

    // Get the value by key. If it does not exist, returns none().
    // docs:start:get
    pub unconstrained fn get<H>(self, key: K) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:get
        let mut result = Option::none();

        let hash = self.hash(key);

        for attempt in 0..self._table.len() {
            let index = self.quadratic_probe(hash, attempt as u32);
            let slot = self._table[index];

            // Not marked as deleted and has key-value.
            if slot.is_valid() {
                let (current_key, value) = slot.key_value_unchecked();
                if current_key == key {
                    result = Option::some(value);
                    break;
                }
            }
        }

        result
    }

    // Insert key-value entry. In case key was already present, value is overridden.
    // docs:start:insert
    pub unconstrained fn insert<H>(&mut self, key: K, value: V)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:insert
        self.try_resize();

        let hash = self.hash(key);

        for attempt in 0..self._table.len() {
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
                break;
            }
        }
    }

    unconstrained fn try_resize<H>(&mut self)
    where
        B: BuildHasher<H>,
        K: Eq + Hash,
        H: Hasher,
    {
        if self.len() + 1 >= self.capacity() / 2 {
            let capacity = self.capacity() * 2;
            let mut new_map = UHashMap::with_hasher_and_capacity(self._build_hasher, capacity);

            for entry in self.entries() {
                new_map.insert(entry.0, entry.1);
            }
            *self = new_map;
        }
    }

    // Removes a key-value entry. If key is not present, UHashMap remains unchanged.
    // docs:start:remove
    pub unconstrained fn remove<H>(&mut self, key: K)
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher,
    {
        // docs:end:remove
        let hash = self.hash(key);

        for attempt in 0..self._table.len() {
            let index = self.quadratic_probe(hash, attempt as u32);
            let mut slot = self._table[index];

            // Not marked as deleted and has key-value.
            if slot.is_valid() {
                let (current_key, _) = slot.key_value_unchecked();
                if current_key == key {
                    slot.mark_deleted();
                    self._table[index] = slot;
                    self._len -= 1;
                    break;
                }
            }
        }
    }

    // Apply UHashMap's hasher onto key to obtain pre-hash for probing.
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
    fn quadratic_probe(self: Self, hash: u32, attempt: u32) -> u32 {
        (hash + (attempt + attempt * attempt) / 2) % self._table.len()
    }
}

// Equality class on UHashMap has to test that they have
// equal sets of key-value entries,
// thus one is a subset of the other and vice versa.
// docs:start:eq
impl<K, V, B, H> Eq for UHashMap<K, V, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher<H>,
    H: Hasher,
{
    fn eq(self, other: UHashMap<K, V, B>) -> bool {
        // docs:end:eq
        let mut equal = false;

        if self.len() == other.len() {
            equal = true;
            for slot in self._table {
                // Not marked as deleted and has key-value.
                if equal & slot.is_valid() {
                    let (key, value) = slot.key_value_unchecked();
                    let other_value = unsafe { other.get(key) };

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
impl<K, V, B, H> Default for UHashMap<K, V, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default,
{
    fn default() -> Self {
        // docs:end:default
        UHashMap::with_hasher(B::default())
    }
}
