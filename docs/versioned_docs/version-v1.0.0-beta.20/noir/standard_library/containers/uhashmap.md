---
title: UHashMap
description: A growable key–value map with a generic hasher.
keywords: [noir, map, hash, hashmap]
sidebar_position: 1
---

`UHashMap<Key, Value, Hasher>` is used to efficiently store and look up key-value pairs in unconstrained
or comptime code.

> Note that the results of most `UHashMap` methods are not constrained! If returning these values into
> constrained code, users must manually ensure they are properly constrained.

`UHashMap` is an unbounded container type implemented with vectors internally which grows as more
entries are pushed to it.

Example:

```rust
// Create a mapping from Fields to u32s using a poseidon2 hasher
use poseidon::poseidon2::Poseidon2Hasher;
let mut map: UHashMap<Field, u32, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

map.insert(1, 2);
map.insert(3, 4);

let two = map.get(1).unwrap();
```

## Methods

### default

```rust title="default" showLineNumbers 
impl<K, V, B> Default for UHashMap<K, V, B>
where
    B: BuildHasher + Default,
{
    fn default() -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L451-L457" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L451-L457</a></sub></sup>


Creates a fresh, empty UHashMap.

This is the same `default` from the `Default` implementation given further below. It is
repeated here for convenience since it is the recommended way to create a hash map.

Example:

```rust title="default_example" showLineNumbers 
let hashmap: UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L205-L208" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L205-L208</a></sub></sup>


Because `UHashMap` has so many generic arguments that are likely to be the same throughout
your program, it may be helpful to create a type alias:

```rust title="type_alias" showLineNumbers 
type MyMap = UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>>;
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L199-L201" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L199-L201</a></sub></sup>


### with_hasher

```rust title="with_hasher" showLineNumbers 
pub fn with_hasher(_build_hasher: B) -> Self
    where
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L72-L77" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L72-L77</a></sub></sup>


Creates a hash map with an existing `BuildHasher`. This can be used to ensure multiple
hash maps are created with the same hasher instance.

Example:

```rust title="with_hasher_example" showLineNumbers 
let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
    let hashmap: UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>> =
        UHashMap::with_hasher(my_hasher);
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L209-L214" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L209-L214</a></sub></sup>


### get

```rust title="get" showLineNumbers 
pub unconstrained fn get(&self, key: K) -> Option<V>
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L275-L281" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L275-L281</a></sub></sup>


Retrieves a value from the hash map, returning `Option::none()` if it was not found.

Example:

```rust title="get_example" showLineNumbers 
fn get_example(map: &UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>>) {
    // Safety: testing context
    let x = unsafe { map.get(12) };

    if x.is_some() {
        assert(x.unwrap() == 42);
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L294-L303" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L294-L303</a></sub></sup>


### insert

```rust title="insert" showLineNumbers 
pub unconstrained fn insert(&mut self, key: K, value: V)
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L303-L309" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L303-L309</a></sub></sup>


Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

```rust title="insert_example" showLineNumbers 
let mut map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    map.insert(12, 42);
    assert(map.len() == 1);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L215-L219" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L215-L219</a></sub></sup>


### remove

```rust title="remove" showLineNumbers 
pub unconstrained fn remove(&mut self, key: K)
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L370-L376" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L370-L376</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L222-L229" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L222-L229</a></sub></sup>


### is_empty

```rust title="is_empty" showLineNumbers 
pub fn is_empty(&self) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L116-L118" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L116-L118</a></sub></sup>


True if the length of the hash map is empty.

Example:

```rust title="is_empty_example" showLineNumbers 
assert(map.is_empty());

    map.insert(1, 2);
    assert(!map.is_empty());

    map.remove(1);
    assert(map.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L230-L238" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L230-L238</a></sub></sup>


### len

```rust title="len" showLineNumbers 
pub fn len(&self) -> u32 {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L261-L263" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L261-L263</a></sub></sup>


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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L239-L254" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L239-L254</a></sub></sup>


### clear

```rust title="clear" showLineNumbers 
pub fn clear(&mut self) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L96-L98" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L96-L98</a></sub></sup>


Clears the hash map, removing all key-value pairs from it.

Example:

```rust title="clear_example" showLineNumbers 
assert(!map.is_empty());
    map.clear();
    assert(map.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L261-L265" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L261-L265</a></sub></sup>


### contains_key

```rust title="contains_key" showLineNumbers 
pub fn contains_key(&self, key: K) -> bool
    where
        K: Hash + Eq,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L104-L110" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L104-L110</a></sub></sup>


True if the hash map contains the given key. Unlike `get`, this will not also return
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
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L266-L273" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L266-L273</a></sub></sup>


### entries

```rust title="entries" showLineNumbers 
pub fn entries(&self) -> [(K, V)] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L124-L126" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L124-L126</a></sub></sup>


Returns a vector of each key-value pair present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

```rust title="entries_example" showLineNumbers 
let entries = map.entries();

    // The length of a hashmap may not be compile-time known, so we
    // need to loop over its capacity instead
    for i in 0..map.capacity() {
        if i < entries.len() {
            let (key, value) = entries[i];
            println(f"{key} -> {value}");
        }
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L306-L317" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L306-L317</a></sub></sup>


### keys

```rust title="keys" showLineNumbers 
pub fn keys(&self) -> [K] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L147-L149" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L147-L149</a></sub></sup>


Returns a vector of each key present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

```rust title="keys_example" showLineNumbers 
let keys = map.keys();

    for key in keys {
        // Safety: testing context
        let value = unsafe { map.get(key) }.unwrap_unchecked();
        println(f"{key} -> {value}");
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L318-L326" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L318-L326</a></sub></sup>


### values

```rust title="values" showLineNumbers 
pub fn values(&self) -> [V] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L169-L171" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L169-L171</a></sub></sup>


Returns a vector of each value present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

```rust title="values_example" showLineNumbers 
let values = map.values();

    for value in values {
        println(f"Found value {value}");
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L327-L333" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L327-L333</a></sub></sup>


### iter_mut

```rust title="iter_mut" showLineNumbers 
pub unconstrained fn iter_mut(&mut self, f: fn(K, V) -> (K, V))
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L190-L196" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L190-L196</a></sub></sup>


Iterates through each key-value pair of the UHashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the UHashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust title="iter_mut_example" showLineNumbers 
// Add 1 to each key in the map, and double the value associated with that key.
    map.iter_mut(|k, v| (k + 1, v * 2));
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L339-L342" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L339-L342</a></sub></sup>


### iter_keys_mut

```rust title="iter_keys_mut" showLineNumbers 
pub unconstrained fn iter_keys_mut(&mut self, f: fn(K) -> K)
    where
        K: Eq + Hash,
        B: BuildHasher,
    {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L210-L216" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L210-L216</a></sub></sup>


Iterates through the UHashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the UHashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust title="iter_keys_mut_example" showLineNumbers 
// Double each key, leaving the value associated with that key untouched
    map.iter_keys_mut(|k| k * 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L343-L346" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L343-L346</a></sub></sup>


### iter_values_mut

```rust title="iter_values_mut" showLineNumbers 
pub fn iter_values_mut(&mut self, f: fn(V) -> V) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L232-L234" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L232-L234</a></sub></sup>


Iterates through the UHashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hash map thus does not need to be reordered.

Example:

```rust title="iter_values_mut_example" showLineNumbers 
// Halve each value
    map.iter_values_mut(|v| v / 2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L347-L350" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L347-L350</a></sub></sup>


### retain

```rust title="retain" showLineNumbers 
pub fn retain(&mut self, f: fn(K, V) -> bool) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L245-L247" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L245-L247</a></sub></sup>


Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

```rust title="retain_example" showLineNumbers 
map.retain(|k, v| (k != 0) & (v != 0));
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L277-L279" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L277-L279</a></sub></sup>


## Trait Implementations

### default

```rust title="default" showLineNumbers 
impl<K, V, B> Default for UHashMap<K, V, B>
where
    B: BuildHasher + Default,
{
    fn default() -> Self {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L451-L457" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L451-L457</a></sub></sup>


Constructs an empty UHashMap.

Example:

```rust title="default_example" showLineNumbers 
let hashmap: UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    assert(hashmap.is_empty());
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L205-L208" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L205-L208</a></sub></sup>


### eq

```rust title="eq" showLineNumbers 
impl<K, V, B> Eq for UHashMap<K, V, B>
where
    K: Eq + Hash,
    V: Eq,
    B: BuildHasher,
{
    fn eq(self, other: UHashMap<K, V, B>) -> bool {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/collections/umap.nr#L416-L424" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/collections/umap.nr#L416-L424</a></sub></sup>


Checks if two UHashMaps are equal.

Example:

```rust title="eq_example" showLineNumbers 
let mut map1: UHashMap<Field, u64, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    let mut map2: UHashMap<Field, u64, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

    map1.insert(1, 2);
    map1.insert(3, 4);

    map2.insert(3, 4);
    map2.insert(1, 2);

    assert(map1 == map2);
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/uhashmap/src/main.nr#L280-L291" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/uhashmap/src/main.nr#L280-L291</a></sub></sup>

