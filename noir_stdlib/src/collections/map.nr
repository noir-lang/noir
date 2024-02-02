use crate::cmp::Eq;
use crate::collections::vec::Vec;
use crate::option::Option;
use crate::default::Default;
use crate::hash::{Hash, Hasher, BuildHasher};

// Generic trait for one-directional key -> value data structure.
// May be implemented by either hash tables or graphs for instance.
trait Map<K, V> {
    fn len(self) -> u64;
    fn get(self, key: K) -> Option<V>;
    fn insert(&mut self, key: K, value: V);
    fn remove(&mut self, key: K);
}

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
// In case Noir would support enums in future, this  
// better to be refactored to use it with three states:
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

impl<K, V> Slot<K, V>{
    fn is_valid(self) -> bool{
        !self._is_deleted & self._key_value.is_some()
    }

    fn is_available(self) -> bool{
        self._is_deleted | self._key_value.is_none()
    }

    fn key_value_unchecked(self) -> (K, V){
        self._key_value.unwrap_unchecked()
    }

    fn set(&mut self, key: K, value: V){
        self._key_value = Option::some((key, value));
        self._is_deleted = false;
    }

    // Shall not override `_key_value` with Option::none(),
    // because we must we able to differentiate empty 
    // and deleted slots for lookup.
    fn mark_deleted(&mut self){
        self._is_deleted = true;
    }
}

impl<K, V, N, B, H> HashMap<K, V, N, B> {   
    // Creates a new instance of HashMap with specified BuildHasher.
    pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher<H>
    {
        let _table = [Slot::default(); N];
        let _len = 0;
        Self{_table, _len, _build_hasher}
    }

    // Clears the map, removing all key-value pairs.
    pub fn clear(&mut self){
        self._table = [Slot::default(); N];
        self._len = 0;
    }

    // Returns true if the map contains a value for the specified key.
    pub fn contains_key(self, key: K) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher<H>,
        H: Hasher
    {
        self.get(key).is_some()
    }

    // Returns true if the map contains no elements.
    pub fn is_empty(self) -> bool{
        self._len == 0
    }

    // Returns a vector of all active keys.
    pub fn keys(self) -> Vec<K>{
        let mut keys = Vec::new();
        for slot in self._table{
            if slot.is_valid(){
                let (key, _) = slot.key_value_unchecked();
                keys.push(key);
            }
        }
        keys
    }

    // Returns a vector of all active values. Not necessarily unique.
    pub fn values(self) -> Vec<V>{
        let mut values = Vec::new();
        for slot in self._table{
            if slot.is_valid(){
                let (_, value) = slot.key_value_unchecked();
                values.push(value);
            }
        }
        values
    }

    // Retains only the elements specified by the predicate.
    pub fn retain(&mut self, f: fn(K, V) -> bool){
        for index in 0..N{
            let mut slot = self._table[index];
            if slot.is_valid(){
                let (key, value) = slot.key_value_unchecked();
                if !f(key, value){
                    slot.mark_deleted();
                    self._len -= 1;
                    self._table[index] = slot;
                }
            }
        }
    }

    fn hash(self, key: K) -> u64
    where
        K: Hash,
        B: BuildHasher<H>,
        H: Hasher
    {
        let mut hasher = self._build_hasher.build_wrapper();
        key.hash(&mut hasher);
        hasher.finish() as u64
    }

    // Probing scheme: quadratic function.
    // We use 0.5 constant near variadic attempt and attempt^2 monomials.
    // This ensures good uniformity of distribution for table sizes
    // equal to prime numbers or powers of two. 
    fn quadratic_probe(self, hash: u64, attempt: u64) -> u64 {
        (hash + (attempt + attempt * attempt) / 2 ) % N
    }
}

// While conducting lookup, we iterate attempt from 0 to N - 1 due to heuristic,
// that if we have went that far without finding desired, 
// it is very unlikely to be after - performance in that case would be drastical.
impl<K, V, N, B, H> Map<K, V> for HashMap<K, V, N, B>
where
    K: Eq + Hash,
    B: BuildHasher<H>,
    H: Hasher
{
    fn len(self) -> u64{
        self._len
    }

    fn get(self, key: K) -> Option<V>{
        let mut result = Option::none();

        let hash = self.hash(key);
        let mut break = false;

        for attempt in 0..N{
            if !break{
                let index = self.quadratic_probe(hash, attempt as u64);
                let slot = self._table[index];

                // Not marked as deleted and has key-value.
                if slot.is_valid(){ 
                    let (current_key, value) = slot.key_value_unchecked();
                    if current_key == key{
                        result = Option::some(value);
                        break = true;
                    }
                }
            }
        }

        result
    }

    fn insert(&mut self, key: K, value: V){
        let hash = self.hash(key);
        let mut break = false;

        for attempt in 0..N{
            if !break{
                let index = self.quadratic_probe(hash, attempt as u64);
                let mut slot = self._table[index];
                let mut insert = false;

                // Either marked as deleted or has unset key-value.
                if slot.is_available(){ 
                    insert = true;
                    self._len += 1;
                }else{
                    let (current_key, _) = slot.key_value_unchecked();
                    if current_key == key{
                        insert = true;
                    }
                }

                if insert{
                    slot.set(key, value);
                    self._table[index] = slot;
                    break = true;
                }
            }
        }
    }

    fn remove(&mut self, key: K){
        let hash = self.hash(key);
        let mut break = false;

        for attempt in 0..N{
            if !break{
                let index = self.quadratic_probe(hash, attempt as u64);
                let mut slot = self._table[index];

                // Not marked as deleted and has key-value.
                if slot.is_valid(){ 
                    let (current_key, _) = slot.key_value_unchecked();
                    if current_key == key{
                        slot.mark_deleted();
                        self._table[index] = slot;
                        self._len -= 1;
                        break = true;
                    }
                }
            }
        }
    }
}

// Equality class on HashMap has to test that they have 
// equal sets of key-value pairs, 
// thus one is a subset of the other and vice versa.
impl<K, V, N, B, H> Eq for HashMap<K, V, N, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher<H>,
    H: Hasher
{
    fn eq(self, other: HashMap<K, V, N, B>) -> bool{
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

impl<K, V, N, B, H> Default for HashMap<K, V, N, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default
{
    fn default() -> Self{
        let _build_hasher = B::default();
        let map: HashMap<K, V, N, B> = HashMap::with_hasher(_build_hasher);
        map
    }
}
