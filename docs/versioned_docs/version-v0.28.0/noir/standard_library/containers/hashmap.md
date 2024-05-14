---
title: HashMap
keywords: [noir, map, hash, hashmap]
sidebar_position: 1
---

`HashMap<Key, Value, MaxLen, Hasher>` is used to efficiently store and look up key-value pairs.

`HashMap` is a bounded type which can store anywhere from zero to `MaxLen` total elements.
Note that due to hash collisions, the actual maximum number of elements stored by any particular
hashmap is likely lower than `MaxLen`. This is true even with cryptographic hash functions since
every hash value will be performed modulo `MaxLen`.

When creating `HashMap`s, the `MaxLen` generic should always be specified if it is not already
known. Otherwise, the compiler may infer a different value for `MaxLen` (such as zero), which
will likely change the result of the program. This behavior is set to become an error in future
versions instead.

Example:

```rust
// Create a mapping from Fields to u32s with a maximum length of 12
// using a poseidon2 hasher
use dep::std::hash::poseidon2::Poseidon2Hasher;
let mut map: HashMap<Field, u32, 12, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();

map.insert(1, 2);
map.insert(3, 4);

let two = map.get(1).unwrap();
```

## Methods

### default

```rust title="default" showLineNumbers 
impl<K, V, N, B, H> Default for HashMap<K, V, N, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default
{
    fn default() -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L462-L469" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L462-L469</a></sub></sup>


Creates a fresh, empty HashMap.

When using this function, always make sure to specify the maximum size of the hash map.

This is the same `default` from the `Default` implementation given further below. It is
repeated here for convenience since it is the recommended way to create a hashmap.

Example:

```rust title="default_example" showLineNumbers 
let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L202-L205" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L202-L205</a></sub></sup>


Because `HashMap` has so many generic arguments that are likely to be the same throughout
your program, it may be helpful to create a type alias:

```rust title="type_alias" showLineNumbers 
type MyMap = HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>>;
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L196-L198" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L196-L198</a></sub></sup>


### with_hasher

```rust title="with_hasher" showLineNumbers 
pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher<H> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L82-L86" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L82-L86</a></sub></sup>


Creates a hashmap with an existing `BuildHasher`. This can be used to ensure multiple
hashmaps are created with the same hasher instance.

Example:

```rust title="with_hasher_example" showLineNumbers 
let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
    let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::with_hasher(my_hasher);
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L207-L211" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L207-L211</a></sub></sup>


### get

```rust title="get" showLineNumbers 
pub fn get(
        self,
        key: K
    ) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L278-L287" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L278-L287</a></sub></sup>


Retrieves a value from the hashmap, returning `Option::none()` if it was not found.

Example:

```rust title="get_example" showLineNumbers 
fn get_example(map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>>) {
    let x = map.get(12);

    if x.is_some() {
        assert(x.unwrap() == 42);
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L299-L307" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L299-L307</a></sub></sup>


### insert

```rust title="insert" showLineNumbers 
pub fn insert(
        &mut self,
        key: K,
        value: V
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L313-L323" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L313-L323</a></sub></sup>


Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

```rust title="insert_example" showLineNumbers 
let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    map.insert(12, 42);
    assert(map.len() == 1);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L213-L217" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L213-L217</a></sub></sup>


### remove

```rust title="remove" showLineNumbers 
pub fn remove(
        &mut self,
        key: K
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L356-L365" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L356-L365</a></sub></sup>


Removes the given key-value pair from the map. If the key was not already present
in the map, this does nothing.

Example:

```rust title="remove_example" showLineNumbers 
map.remove(12);
    assert(map.is_empty());

    // If a key was not present in the map, remove does nothing
    map.remove(12);
    assert(map.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L221-L228" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L221-L228</a></sub></sup>


### is_empty

```rust title="is_empty" showLineNumbers 
pub fn is_empty(self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L115-L117" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L115-L117</a></sub></sup>


True if the length of the hash map is empty.

Example:

```rust title="is_empty_example" showLineNumbers 
assert(map.is_empty());

    map.insert(1, 2);
    assert(!map.is_empty());

    map.remove(1);
    assert(map.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L230-L238" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L230-L238</a></sub></sup>


### len

```rust title="len" showLineNumbers 
pub fn len(self) -> u64 {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L264-L266" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L264-L266</a></sub></sup>


Returns the current length of this hash map.

Example:

```rust title="len_example" showLineNumbers 
// This is equivalent to checking map.is_empty()
    assert(map.len() == 0);

    map.insert(1, 2);
    map.insert(3, 4);
    map.insert(5, 6);
    assert(map.len() == 3);

    // 3 was already present as a key in the hash map, so the length is unchanged
    map.insert(3, 7);
    assert(map.len() == 3);

    map.remove(1);
    assert(map.len() == 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L240-L255" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L240-L255</a></sub></sup>


### capacity

```rust title="capacity" showLineNumbers 
pub fn capacity(_self: Self) -> u64 {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L271-L273" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L271-L273</a></sub></sup>


Returns the maximum capacity of this hashmap. This is always equal to the capacity
specified in the hashmap's type.

Unlike hashmaps in general purpose programming languages, hashmaps in Noir have a
static capacity that does not increase as the map grows larger. Thus, this capacity
is also the maximum possible element count that can be inserted into the hashmap.
Due to hash collisions (modulo the hashmap length), it is likely the actual maximum
element count will be lower than the full capacity.

Example:

```rust title="capacity_example" showLineNumbers 
let empty_map: HashMap<Field, Field, 42, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    assert(empty_map.len() == 0);
    assert(empty_map.capacity() == 42);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L257-L261" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L257-L261</a></sub></sup>


### clear

```rust title="clear" showLineNumbers 
pub fn clear(&mut self) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L93-L95" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L93-L95</a></sub></sup>


Clears the hashmap, removing all key-value pairs from it.

Example:

```rust title="clear_example" showLineNumbers 
assert(!map.is_empty());
    map.clear();
    assert(map.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L263-L267" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L263-L267</a></sub></sup>


### contains_key

```rust title="contains_key" showLineNumbers 
pub fn contains_key(
        self,
        key: K
    ) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L101-L110" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L101-L110</a></sub></sup>


True if the hashmap contains the given key. Unlike `get`, this will not also return
the value associated with the key.

Example:

```rust title="contains_key_example" showLineNumbers 
if map.contains_key(7) {
        let value = map.get(7);
        assert(value.is_some());
    } else {
        println("No value for key 7!");
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L269-L276" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L269-L276</a></sub></sup>


### entries

```rust title="entries" showLineNumbers 
pub fn entries(self) -> BoundedVec<(K, V), N> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L123-L125" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L123-L125</a></sub></sup>


Returns a vector of each key-value pair present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust title="entries_example" showLineNumbers 
let entries = map.entries();

    // The length of a hashmap may not be compile-time known, so we
    // need to loop over its capacity instead
    for i in 0..map.capacity() {
        if i < entries.len() {
            let (key, value) = entries.get(i);
            println(f"{key} -> {value}");
        }
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L310-L321" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L310-L321</a></sub></sup>


### keys

```rust title="keys" showLineNumbers 
pub fn keys(self) -> BoundedVec<K, N> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L144-L146" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L144-L146</a></sub></sup>


Returns a vector of each key present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust title="keys_example" showLineNumbers 
let keys = map.keys();

    for i in 0..keys.max_len() {
        if i < keys.len() {
            let key = keys.get_unchecked(i);
            let value = map.get(key).unwrap_unchecked();
            println(f"{key} -> {value}");
        }
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L323-L333" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L323-L333</a></sub></sup>


### values

```rust title="values" showLineNumbers 
pub fn values(self) -> BoundedVec<V, N> {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L164-L166" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L164-L166</a></sub></sup>


Returns a vector of each value present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust title="values_example" showLineNumbers 
let values = map.values();

    for i in 0..values.max_len() {
        if i < values.len() {
            let value = values.get_unchecked(i);
            println(f"Found value {value}");
        }
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L335-L344" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L335-L344</a></sub></sup>


### iter_mut

```rust title="iter_mut" showLineNumbers 
pub fn iter_mut(
        &mut self,
        f: fn(K, V) -> (K, V)
    )
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L183-L192" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L183-L192</a></sub></sup>


Iterates through each key-value pair of the HashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust title="iter_mut_example" showLineNumbers 
// Add 1 to each key in the map, and double the value associated with that key.
    map.iter_mut(|k, v| (k + 1, v * 2));
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L348-L351" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L348-L351</a></sub></sup>


### iter_keys_mut

```rust title="iter_keys_mut" showLineNumbers 
pub fn iter_keys_mut(
        &mut self,
        f: fn(K) -> K
    ) 
    where
        K: Eq + Hash,
        B: BuildHasher<H>,
        H: Hasher {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L208-L217" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L208-L217</a></sub></sup>


Iterates through the HashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust title="iter_keys_mut_example" showLineNumbers 
// Double each key, leaving the value associated with that key untouched
    map.iter_keys_mut(|k| k * 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L353-L356" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L353-L356</a></sub></sup>


### iter_values_mut

```rust title="iter_values_mut" showLineNumbers 
pub fn iter_values_mut(&mut self, f: fn(V) -> V) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L233-L235" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L233-L235</a></sub></sup>


Iterates through the HashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hashmap thus does not need to be reordered.

Example:

```rust title="iter_values_mut_example" showLineNumbers 
// Halve each value
    map.iter_values_mut(|v| v / 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L358-L361" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L358-L361</a></sub></sup>


### retain

```rust title="retain" showLineNumbers 
pub fn retain(&mut self, f: fn(K, V) -> bool) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L247-L249" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L247-L249</a></sub></sup>


Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

```rust title="retain_example" showLineNumbers 
map.retain(|k, v| (k != 0) & (v != 0));
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L281-L283" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L281-L283</a></sub></sup>


## Trait Implementations

### default

```rust title="default" showLineNumbers 
impl<K, V, N, B, H> Default for HashMap<K, V, N, B>
where
    B: BuildHasher<H> + Default,
    H: Hasher + Default
{
    fn default() -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L462-L469" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L462-L469</a></sub></sup>


Constructs an empty HashMap.

Example:

```rust title="default_example" showLineNumbers 
let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L202-L205" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L202-L205</a></sub></sup>


### eq

```rust title="eq" showLineNumbers 
impl<K, V, N, B, H> Eq for HashMap<K, V, N, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher<H>,
    H: Hasher
{
    fn eq(self, other: HashMap<K, V, N, B>) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/map.nr#L426-L435" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/map.nr#L426-L435</a></sub></sup>


Checks if two HashMaps are equal.

Example:

```rust title="eq_example" showLineNumbers 
let mut map1: HashMap<Field, u64, 4, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
    let mut map2: HashMap<Field, u64, 4, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();

    map1.insert(1, 2);
    map1.insert(3, 4);

    map2.insert(3, 4);
    map2.insert(1, 2);

    assert(map1 == map2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/hashmap/src/main.nr#L285-L296" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/hashmap/src/main.nr#L285-L296</a></sub></sup>

