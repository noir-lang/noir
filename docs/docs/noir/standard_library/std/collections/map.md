---
title: map
---

# Module `std::collections::map`

## Struct `HashMap<K, V, let N: u32, B>`

`HashMap<Key, Value, MaxLen, Hasher>` is used to efficiently store and look up key-value pairs.

`HashMap` is a bounded type which can store anywhere from zero to `MaxLen` total elements.
Note that due to hash collisions, the actual maximum number of elements stored by any particular
hashmap is likely lower than `MaxLen`. This is true even with cryptographic hash functions since
every hash value will be performed modulo `MaxLen`.

Example:

```rust
// Create a mapping from Fields to u32s with a maximum length of 12
// using a poseidon2 hasher
use std::hash::poseidon2::Poseidon2Hasher;
let mut map: HashMap<Field, u32, 12, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();

map.insert(1, 2);
map.insert(3, 4);

let two = map.get(1).unwrap();
```

### Methods

#### with_hasher

```rust
fn with_hasher<H>(_build_hasher: B) -> Self
    where B: BuildHasher<H>
```

Creates a hashmap with an existing `BuildHasher`. This can be used to ensure multiple
hashmaps are created with the same hasher instance.

Example:

```rust
let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::with_hasher(my_hasher);
assert(hashmap.is_empty());
```

#### clear

```rust
fn clear(self)
```

Clears the hashmap, removing all key-value pairs from it.

Example:

```rust
assert(!map.is_empty());
map.clear();
assert(map.is_empty());
```

#### contains_key

```rust
fn contains_key<H>(self, key: K) -> bool
    where K: Hash,
          K: Eq,
          B: BuildHasher<H>,
          H: Hasher
```

Returns `true` if the hashmap contains the given key. Unlike `get`, this will not also return
the value associated with the key.

Example:

```rust
if map.contains_key(7) {
let value = map.get(7);
assert(value.is_some());
} else {
println("No value for key 7!");
}
```

#### is_empty

```rust
fn is_empty(self) -> bool
```

Returns `true` if the length of the hash map is empty.

Example:

```rust
assert(map.is_empty());

map.insert(1, 2);
assert(!map.is_empty());

map.remove(1);
assert(map.is_empty());
```

#### entries

```rust
fn entries(self) -> BoundedVec<(K, V), N>
```

Returns a vector of each key-value pair present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust
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

#### keys

```rust
fn keys(self) -> BoundedVec<K, N>
```

Returns a vector of each key present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust
let keys = map.keys();

for i in 0..keys.max_len() {
if i < keys.len() {
let key = keys.get_unchecked(i);
let value = map.get(key).unwrap_unchecked();
println(f"{key} -> {value}");
}
}
```

#### values

```rust
fn values(self) -> BoundedVec<V, N>
```

Returns a vector of each value present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```rust
let values = map.values();

for i in 0..values.max_len() {
if i < values.len() {
let value = values.get_unchecked(i);
println(f"Found value {value}");
}
}
```

#### iter_mut

```rust
fn iter_mut<H>(self, f: fn(K, V) -> (K, V))
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

Iterates through each key-value pair of the HashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust
// Add 1 to each key in the map, and double the value associated with that key.
map.iter_mut(|k, v| (k + 1, v * 2));
```

#### iter_keys_mut

```rust
fn iter_keys_mut<H>(self, f: fn(K) -> K)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

Iterates through the HashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```rust
// Double each key, leaving the value associated with that key untouched
map.iter_keys_mut(|k| k * 2);
```

#### iter_values_mut

```rust
fn iter_values_mut(self, f: fn(V) -> V)
```

Iterates through the HashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hashmap thus does not need to be reordered.

Example:

```rust
// Halve each value
map.iter_values_mut(|v| v / 2);
```

#### retain

```rust
fn retain(self, f: fn(K, V) -> bool)
```

Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

```rust
map.retain(|k, v| (k != 0) & (v != 0));
```

#### len

```rust
fn len(self) -> u32
```

Returns the current length of this hash map.

Example:

```rust
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

#### capacity

```rust
fn capacity(_self) -> u32
```

Returns the maximum capacity of this hashmap. This is always equal to the capacity
specified in the hashmap's type.

Unlike hashmaps in general purpose programming languages, hashmaps in Noir have a
static capacity that does not increase as the map grows larger. Thus, this capacity
is also the maximum possible element count that can be inserted into the hashmap.
Due to hash collisions (modulo the hashmap length), it is likely the actual maximum
element count will be lower than the full capacity.

Example:

```rust
let empty_map: HashMap<Field, Field, 42, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
assert(empty_map.len() == 0);
assert(empty_map.capacity() == 42);
```

#### get

```rust
fn get<H>(self, key: K) -> Option<V>
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

Retrieves a value from the hashmap, returning `Option::none()` if it was not found.

Example:

```rust
fn get_example(map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>>) {
let x = map.get(12);

if x.is_some() {
assert(x.unwrap() == 42);
}
}
```

#### insert

```rust
fn insert<H>(self, key: K, value: V)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

```rust
let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
map.insert(12, 42);
assert(map.len() == 1);
```

#### remove

```rust
fn remove<H>(self, key: K)
    where K: Eq,
          K: Hash,
          B: BuildHasher<H>,
          H: Hasher
```

Removes the given key-value pair from the map. If the key was not already present
in the map, this does nothing.

Example:

```rust
let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
map.insert(12, 42);
assert(!map.is_empty());

map.remove(12);
assert(map.is_empty());

// If a key was not present in the map, remove does nothing
map.remove(12);
assert(map.is_empty());
```

