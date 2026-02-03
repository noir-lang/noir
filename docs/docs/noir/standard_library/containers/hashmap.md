---
title: HashMap
description: A bounded key–value map with fixed capacity and Poseidon-compatible hashing—APIs for insert, get, iteration, and more.
keywords: [noir, map, hash, hashmap]
sidebar_position: 1
---

`HashMap<Key, Value, MaxLen, Hasher>` is used to efficiently store and look up key-value pairs.

`HashMap` is a bounded type which can store anywhere from zero to `MaxLen` total elements.
Note that due to hash collisions, the actual maximum number of elements stored by any particular
hashmap is likely lower than `MaxLen`. This is true even with cryptographic hash functions since
every hash value will be performed modulo `MaxLen`.

Example:

```rust
// Create a mapping from Fields to u32s with a maximum length of 12
// using a poseidon2 hasher
use poseidon::poseidon2::Poseidon2Hasher;
let mut map: HashMap<Field, u32, 12, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();

map.insert(1, 2);
map.insert(3, 4);

let two = map.get(1).unwrap();
```

## Methods

### default

#include_code default noir_stdlib/src/collections/map.nr rust

Creates a fresh, empty HashMap.

When using this function, always make sure to specify the maximum size of the hash map.

This is the same `default` from the `Default` implementation given further below. It is
repeated here for convenience since it is the recommended way to create a hashmap.

Example:

#include_code default_example test_programs/execution_success/hashmap/src/main.nr rust

Because `HashMap` has so many generic arguments that are likely to be the same throughout
your program, it may be helpful to create a type alias:

#include_code type_alias test_programs/execution_success/hashmap/src/main.nr rust

### with_hasher

#include_code with_hasher noir_stdlib/src/collections/map.nr rust

Creates a hashmap with an existing `BuildHasher`. This can be used to ensure multiple
hashmaps are created with the same hasher instance.

Example:

#include_code with_hasher_example test_programs/execution_success/hashmap/src/main.nr rust

### get

#include_code get noir_stdlib/src/collections/map.nr rust

Retrieves a value from the hashmap, returning `Option::none()` if it was not found.

Example:

#include_code get_example test_programs/execution_success/hashmap/src/main.nr rust

### insert

#include_code insert noir_stdlib/src/collections/map.nr rust

Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

#include_code insert_example test_programs/execution_success/hashmap/src/main.nr rust

### remove

#include_code remove noir_stdlib/src/collections/map.nr rust

Removes the given key-value pair from the map. If the key was not already present
in the map, this does nothing.

Example:

#include_code remove_example test_programs/execution_success/hashmap/src/main.nr rust

### is_empty

#include_code is_empty noir_stdlib/src/collections/map.nr rust

True if the length of the hash map is empty.

Example:

#include_code is_empty_example test_programs/execution_success/hashmap/src/main.nr rust

### len

#include_code len noir_stdlib/src/collections/map.nr rust

Returns the current length of this hash map.

Example:

#include_code len_example test_programs/execution_success/hashmap/src/main.nr rust

### capacity

#include_code capacity noir_stdlib/src/collections/map.nr rust

Returns the maximum capacity of this hashmap. This is always equal to the capacity
specified in the hashmap's type.

Unlike hashmaps in general purpose programming languages, hashmaps in Noir have a
static capacity that does not increase as the map grows larger. Thus, this capacity
is also the maximum possible element count that can be inserted into the hashmap.
Due to hash collisions (modulo the hashmap length), it is likely the actual maximum
element count will be lower than the full capacity.

Example:

#include_code capacity_example test_programs/execution_success/hashmap/src/main.nr rust

### clear

#include_code clear noir_stdlib/src/collections/map.nr rust

Clears the hashmap, removing all key-value pairs from it.

Example:

#include_code clear_example test_programs/execution_success/hashmap/src/main.nr rust

### contains_key

#include_code contains_key noir_stdlib/src/collections/map.nr rust

True if the hashmap contains the given key. Unlike `get`, this will not also return
the value associated with the key.

Example:

#include_code contains_key_example test_programs/execution_success/hashmap/src/main.nr rust

### entries

#include_code entries noir_stdlib/src/collections/map.nr rust

Returns a vector of each key-value pair present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

#include_code entries_example test_programs/execution_success/hashmap/src/main.nr rust

### keys

#include_code keys noir_stdlib/src/collections/map.nr rust

Returns a vector of each key present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

#include_code keys_example test_programs/execution_success/hashmap/src/main.nr rust

### values

#include_code values noir_stdlib/src/collections/map.nr rust

Returns a vector of each value present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

#include_code values_example test_programs/execution_success/hashmap/src/main.nr rust

### iter_mut

#include_code iter_mut noir_stdlib/src/collections/map.nr rust

Iterates through each key-value pair of the HashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

#include_code iter_mut_example test_programs/execution_success/hashmap/src/main.nr rust

### iter_keys_mut

#include_code iter_keys_mut noir_stdlib/src/collections/map.nr rust

Iterates through the HashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

#include_code iter_keys_mut_example test_programs/execution_success/hashmap/src/main.nr rust

### iter_values_mut

#include_code iter_values_mut noir_stdlib/src/collections/map.nr rust

Iterates through the HashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hashmap thus does not need to be reordered.

Example:

#include_code iter_values_mut_example test_programs/execution_success/hashmap/src/main.nr rust

### retain

#include_code retain noir_stdlib/src/collections/map.nr rust

Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

#include_code retain_example test_programs/execution_success/hashmap/src/main.nr rust

## Trait Implementations

### default

#include_code default noir_stdlib/src/collections/map.nr rust

Constructs an empty HashMap.

Example:

#include_code default_example test_programs/execution_success/hashmap/src/main.nr rust

### eq

#include_code eq noir_stdlib/src/collections/map.nr rust

Checks if two HashMaps are equal.

Example:

#include_code eq_example test_programs/execution_success/hashmap/src/main.nr rust
