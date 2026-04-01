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

#include_code default noir_stdlib/src/collections/umap.nr rust

Creates a fresh, empty UHashMap.

This is the same `default` from the `Default` implementation given further below. It is
repeated here for convenience since it is the recommended way to create a hash map.

Example:

#include_code default_example test_programs/execution_success/uhashmap/src/main.nr rust

Because `UHashMap` has so many generic arguments that are likely to be the same throughout
your program, it may be helpful to create a type alias:

#include_code type_alias test_programs/execution_success/uhashmap/src/main.nr rust

### with_hasher

#include_code with_hasher noir_stdlib/src/collections/map.nr rust

Creates a hash map with an existing `BuildHasher`. This can be used to ensure multiple
hash maps are created with the same hasher instance.

Example:

#include_code with_hasher_example test_programs/execution_success/uhashmap/src/main.nr rust

### get

#include_code get noir_stdlib/src/collections/map.nr rust

Retrieves a value from the hash map, returning `Option::none()` if it was not found.

Example:

#include_code get_example test_programs/execution_success/uhashmap/src/main.nr rust

### insert

#include_code insert noir_stdlib/src/collections/map.nr rust

Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

#include_code insert_example test_programs/execution_success/uhashmap/src/main.nr rust

### remove

#include_code remove noir_stdlib/src/collections/map.nr rust

Removes the given key-value pair from the map. If the key was not already present
in the map, this does nothing.

Example:

#include_code remove_example test_programs/execution_success/uhashmap/src/main.nr rust

### is_empty

#include_code is_empty noir_stdlib/src/collections/map.nr rust

True if the length of the hash map is empty.

Example:

#include_code is_empty_example test_programs/execution_success/uhashmap/src/main.nr rust

### len

#include_code len noir_stdlib/src/collections/map.nr rust

Returns the current length of this hash map.

Example:

#include_code len_example test_programs/execution_success/uhashmap/src/main.nr rust

### clear

#include_code clear noir_stdlib/src/collections/map.nr rust

Clears the hash map, removing all key-value pairs from it.

Example:

#include_code clear_example test_programs/execution_success/uhashmap/src/main.nr rust

### contains_key

#include_code contains_key noir_stdlib/src/collections/map.nr rust

True if the hash map contains the given key. Unlike `get`, this will not also return
the value associated with the key.

Example:

#include_code contains_key_example test_programs/execution_success/uhashmap/src/main.nr rust

### entries

#include_code entries noir_stdlib/src/collections/map.nr rust

Returns a vector of each key-value pair present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

#include_code entries_example test_programs/execution_success/uhashmap/src/main.nr rust

### keys

#include_code keys noir_stdlib/src/collections/map.nr rust

Returns a vector of each key present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

#include_code keys_example test_programs/execution_success/uhashmap/src/main.nr rust

### values

#include_code values noir_stdlib/src/collections/map.nr rust

Returns a vector of each value present in the hash map.

The length of the returned vector is always equal to the length of the hash map.

Example:

#include_code values_example test_programs/execution_success/uhashmap/src/main.nr rust

### iter_mut

#include_code iter_mut noir_stdlib/src/collections/map.nr rust

Iterates through each key-value pair of the UHashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the UHashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

#include_code iter_mut_example test_programs/execution_success/uhashmap/src/main.nr rust

### iter_keys_mut

#include_code iter_keys_mut noir_stdlib/src/collections/map.nr rust

Iterates through the UHashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the UHashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

#include_code iter_keys_mut_example test_programs/execution_success/uhashmap/src/main.nr rust

### iter_values_mut

#include_code iter_values_mut noir_stdlib/src/collections/map.nr rust

Iterates through the UHashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hash map thus does not need to be reordered.

Example:

#include_code iter_values_mut_example test_programs/execution_success/uhashmap/src/main.nr rust

### retain

#include_code retain noir_stdlib/src/collections/map.nr rust

Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

#include_code retain_example test_programs/execution_success/uhashmap/src/main.nr rust

## Trait Implementations

### default

#include_code default noir_stdlib/src/collections/map.nr rust

Constructs an empty UHashMap.

Example:

#include_code default_example test_programs/execution_success/uhashmap/src/main.nr rust

### eq

#include_code eq noir_stdlib/src/collections/map.nr rust

Checks if two UHashMaps are equal.

Example:

#include_code eq_example test_programs/execution_success/uhashmap/src/main.nr rust
