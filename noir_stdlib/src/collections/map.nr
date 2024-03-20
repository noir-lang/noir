use crate::cmp::Eq;
use crate::collections::vec::Vec;
use crate::option::Option;
use crate::default::Default;
use crate::hash::{Hash, Hasher, BuildHasher};
use crate::collections::bounded_vec::BoundedVec;

// We use load factor α_max = 0.75. 
// Upon exceeding it, assert will fail in order to inform the user 
// about performance degradation, so that he can adjust the capacity.
global MAX_LOAD_FACTOR_NUMERATOR = 3;
global MAX_LOAD_FACTOR_DEN0MINATOR = 4;

// Hash table with open addressing and quadratic probing.
// Size of the underlying table must be known at compile time.
// It is advised to select capacity N as a power of two, or a prime number 
// because utilized probing scheme is best tailored for it.
struct HashMap<K, V, N, B> {
    _table: [Slot<K, V>; N],

    // Amount of valid elements in the map.
    _len: u64,

    _build_hasher: B
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

impl<K, V> Default for Slot<K, V>{
    fn default() -> Self{
        Slot{
            _key_value: Option::none(),
            _is_deleted: false
        }
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
impl<K, V, N, B, H> HashMap<K, V, N, B> {
    // Creates a new instance of HashMap with specified BuildHasher.
    // docs:start:with_hasher
    pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher<H> {
        // docs:end:with_hasher
        let _table = [Slot::default(); N];
        let _len = 0;
        Self { _table, _len, _build_hasher }
    }

    // Clears the map, removing all key-value entries.
    // docs:start:clear
    pub fn clear(&mut self) {
        // docs:end:clear
        self._table = [Slot::default(); N];
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

    // Returns a BoundedVec of all valid entries in this HashMap.
    // The length of the returned vector will always match the length of this HashMap.
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

        let msg = f"Amount of valid elements should have been {self._len} times, but got {entries.len()}.";
        assert(entries.len() == self._len, msg);

        entries
    }

    // Returns a BoundedVec containing all the keys within this HashMap.
    // The length of the returned vector will always match the length of this HashMap.
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

        let msg = f"Amount of valid elements should have been {self._len} times, but got {keys.len()}.";
        assert(keys.len() == self._len, msg);

        keys
    }

    // Returns a BoundedVec containing all the values within this HashMap.
    // The length of the returned vector will always match the length of this HashMap.
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

        let msg = f"Amount of valid elements should have been {self._len} times, but got {values.len()}.";
        assert(values.len() == self._len, msg);

        values
    }

    // For each key-value entry applies mutator function.
    // docs:start:iter_mut
    pub fn iter_mut(
        &mut self,
        f: fn(K, V) -> (K, V)
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
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

    // For each key applies mutator function.
    // docs:start:iter_keys_mut
    pub fn iter_keys_mut(
        &mut self,
        f: fn(K) -> K
    ) 
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
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

    // For each value applies mutator function.
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

    // Retains only the elements specified by the predicate.
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
        let mut result = Option::none();

        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u64);
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
        self.assert_load_factor();

        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u64);
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
        let hash = self.hash(key);
        let mut should_break = false;

        for attempt in 0..N {
            if !should_break {
                let index = self.quadratic_probe(hash, attempt as u64);
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

    // Probing scheme: quadratic function.
    // We use 0.5 constant near variadic attempt and attempt^2 monomials.
    // This ensures good uniformity of distribution for table sizes
    // equal to prime numbers or powers of two. 
    fn quadratic_probe(_self: Self, hash: u64, attempt: u64) -> u64 {
        (hash + (attempt + attempt * attempt) / 2) % N
    }

    // Amount of elements in the table in relation to available slots exceeds α_max. 
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

        if self.len() == other.len(){
            equal = true;
            for slot in self._table{
                // Not marked as deleted and has key-value.
                if equal & slot.is_valid(){
                    let (key, value) = slot.key_value_unchecked();
                    let other_value = other.get(key);

                    if other_value.is_none(){
                        equal = false;
                    }else{
                        let other_value = other_value.unwrap_unchecked();
                        if value != other_value{
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
