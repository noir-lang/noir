use std::collections::umap::UHashMap;
use std::hash::BuildHasherDefault;
use std::hash::poseidon2::Poseidon2Hasher;

type K = Field;
type V = Field;

// It is more convenient and readable to use structs as input.
struct Entry {
    key: Field,
    value: Field,
}

global HASHMAP_LEN: u32 = 6;

global FIELD_CMP: fn(Field, Field) -> bool = |a: Field, b: Field| a.lt(b);

global K_CMP: fn(Field, Field) -> bool = FIELD_CMP;
global V_CMP: fn(Field, Field) -> bool = FIELD_CMP;
global KV_CMP: fn((K, V), (K, V)) -> bool = |a: (K, V), b: (K, V)| a.0.lt(b.0);

global ALLOCATE_HASHMAP: fn() -> UHashMap<K, V, BuildHasherDefault<Poseidon2Hasher>> =
    || -> UHashMap<K, V, BuildHasherDefault<Poseidon2Hasher>> UHashMap::default();

unconstrained fn main(input: [Entry; HASHMAP_LEN]) {
    test_sequential(input[0].key, input[0].value);
    test_multiple_equal_insert(input[1].key, input[1].value);
    test_value_override(input[2].key, input[2].value, input[3].value);
    test_insert_and_methods(input);
    test_hashmaps_equality(input);
    test_retain();
    test_iterators();
    test_mut_iterators();

    doc_tests();
}

// Insert, get, remove.
unconstrained fn test_sequential(key: K, value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New UHashMap should be empty.");

    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "UHashMap after one insert should have a length of 1 element.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(value == got, f"Inserted {value} but got {got} for the same key.");

    hashmap.remove(key);
    assert(
        hashmap.is_empty(),
        "UHashMap after one insert and corresponding removal should be empty.",
    );
    let got = hashmap.get(key);
    assert(got.is_none(), "Value has been removed, but is still available (not none).");
}

// Insert same pair several times.
unconstrained fn test_multiple_equal_insert(key: K, value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New UHashMap should be empty.");

    for _ in 0..HASHMAP_LEN {
        hashmap.insert(key, value);
    }

    let len = hashmap.len();
    assert(len == 1, f"UHashMap length must be 1, got {len}.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(value == got, f"Inserted {value} but got {got} for the same key.");
}

// Override value for existing pair.
unconstrained fn test_value_override(key: K, value: V, new_value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New hashmap should be empty.");

    hashmap.insert(key, value);
    hashmap.insert(key, new_value);
    assert(hashmap.len() == 1, "UHashMap length is invalid.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(got == new_value, f"Expected {new_value}, but got {got}.");
}

// Insert several distinct pairs and test auxiliary methods.
unconstrained fn test_insert_and_methods(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New UHashMap should be empty.");

    for entry in input {
        println(f"Inserting {entry}");
        hashmap.insert(entry.key, entry.value);
    }

    println(hashmap.len());
    assert(hashmap.len() == HASHMAP_LEN, "hashmap.len() does not match input length.");

    for entry in input {
        let entry_key = entry.key;
        assert(hashmap.contains_key(entry.key), f"Not found inserted key {entry_key}.");
    }

    hashmap.clear();
    assert(hashmap.is_empty(), "UHashMap after clear() should be empty.");
}

// Insert several pairs and test retaining.
unconstrained fn test_retain() {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New UHashMap should be empty.");

    let (key, value) = (5, 11);
    hashmap.insert(key, value);
    let (key, value) = (2, 13);
    hashmap.insert(key, value);
    let (key, value) = (11, 5);
    hashmap.insert(key, value);

    let predicate = |key: K, value: V| -> bool { key * value == 55 };
    hashmap.retain(predicate);

    assert(hashmap.len() == 2, "UHashMap should have retained 2 elements.");
    assert(
        hashmap.get(2).is_none(),
        "Pair should have been removed, since it does not match predicate.",
    );
}

// Equality trait check.
unconstrained fn test_hashmaps_equality(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap_1 = ALLOCATE_HASHMAP();
    let mut hashmap_2 = ALLOCATE_HASHMAP();

    for entry in input {
        hashmap_1.insert(entry.key, entry.value);
        hashmap_2.insert(entry.key, entry.value);
    }

    assert(hashmap_1 == hashmap_2, "CtHashMaps should be equal.");

    hashmap_2.remove(input[0].key);

    assert(hashmap_1 != hashmap_2, "CtHashMaps should not be equal.");
}

// Test entries, keys, values.
unconstrained fn test_iterators() {
    let mut hashmap = ALLOCATE_HASHMAP();

    hashmap.insert(2, 3);
    hashmap.insert(5, 7);
    hashmap.insert(11, 13);

    let keys: [K; 3] = hashmap.keys().as_array().sort_via(K_CMP);
    let values: [V; 3] = hashmap.values().as_array().sort_via(V_CMP);
    let entries: [(K, V); 3] = hashmap.entries().as_array().sort_via(KV_CMP);

    assert(keys == [2, 5, 11], "Got incorrect iteration of keys.");
    assert(values == [3, 7, 13], "Got incorrect iteration of values.");
    assert(entries == [(2, 3), (5, 7), (11, 13)], "Got incorrect iteration of entries.");
}

// Test mutable iteration over keys, values and entries.
unconstrained fn test_mut_iterators() {
    let mut hashmap = ALLOCATE_HASHMAP();

    hashmap.insert(2, 3);
    hashmap.insert(5, 7);
    hashmap.insert(11, 13);

    let f = |k: K| -> K { k * 3 };
    hashmap.iter_keys_mut(f);

    let f = |v: V| -> V { v * 5 };
    hashmap.iter_values_mut(f);

    let keys: [K; 3] = hashmap.keys().as_array().sort_via(K_CMP);
    let values: [V; 3] = hashmap.values().as_array().sort_via(V_CMP);

    assert(keys == [6, 15, 33], f"Got incorrect iteration of keys: {keys}");
    assert(values == [15, 35, 65], "Got incorrect iteration of values.");

    let f = |k: K, v: V| -> (K, V) { (k * 2, v * 2) };
    hashmap.iter_mut(f);

    let entries: [(K, V); 3] = hashmap.entries().as_array().sort_via(KV_CMP);

    assert(entries == [(12, 30), (30, 70), (66, 130)], "Got incorrect iteration of entries.");
}

// docs:start:type_alias
type MyMap = UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>>;
// docs:end:type_alias

/// Tests examples from the stdlib cthashmap documentation
unconstrained fn doc_tests() {
    // docs:start:default_example
    let hashmap: UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    assert(hashmap.is_empty());
    // docs:end:default_example
    // docs:start:with_hasher_example
    let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
    let hashmap: UHashMap<u8, u32, BuildHasherDefault<Poseidon2Hasher>> =
        UHashMap::with_hasher(my_hasher);
    assert(hashmap.is_empty());
    // docs:end:with_hasher_example
    // docs:start:insert_example
    let mut map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    map.insert(12, 42);
    assert(map.len() == 1);
    // docs:end:insert_example
    get_example(map);

    // docs:start:remove_example
    map.remove(12);
    assert(map.is_empty());

    // If a key was not present in the map, remove does nothing
    map.remove(12);
    assert(map.is_empty());
    // docs:end:remove_example
    // docs:start:is_empty_example
    assert(map.is_empty());

    map.insert(1, 2);
    assert(!map.is_empty());

    map.remove(1);
    assert(map.is_empty());
    // docs:end:is_empty_example
    // docs:start:len_example
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
    // docs:end:len_example
    // docs:start:capacity_example
    let empty_map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>> =
        UHashMap::default();
    assert(empty_map.len() == 0);
    println(empty_map.capacity());
    // docs:end:capacity_example
    // docs:start:clear_example
    assert(!map.is_empty());
    map.clear();
    assert(map.is_empty());
    // docs:end:clear_example
    // docs:start:contains_key_example
    if map.contains_key(7) {
        let value = map.get(7);
        assert(value.is_some());
    } else {
        println("No value for key 7!");
    }
    // docs:end:contains_key_example
    entries_examples(map);
    iter_examples(map);

    // docs:start:retain_example
    map.retain(|k, v| (k != 0) & (v != 0));
    // docs:end:retain_example
    // docs:start:eq_example
    let mut map1: UHashMap<Field, u64, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();
    let mut map2: UHashMap<Field, u64, BuildHasherDefault<Poseidon2Hasher>> = UHashMap::default();

    map1.insert(1, 2);
    map1.insert(3, 4);

    map2.insert(3, 4);
    map2.insert(1, 2);

    assert(map1 == map2);
    // docs:end:eq_example
}

// docs:start:get_example
fn get_example(map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>>) {
    /// Safety: testing context
    let x = unsafe { map.get(12) };

    if x.is_some() {
        assert(x.unwrap() == 42);
    }
}
// docs:end:get_example

fn entries_examples(map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>>) {
    // docs:start:entries_example
    let entries = map.entries();

    // The length of a hashmap may not be compile-time known, so we
    // need to loop over its capacity instead
    for i in 0..map.capacity() {
        if i < entries.len() {
            let (key, value) = entries[i];
            println(f"{key} -> {value}");
        }
    }
    // docs:end:entries_example
    // docs:start:keys_example
    let keys = map.keys();

    for key in keys {
        /// Safety: testing context
        let value = unsafe { map.get(key) }.unwrap_unchecked();
        println(f"{key} -> {value}");
    }
    // docs:end:keys_example
    // docs:start:values_example
    let values = map.values();

    for value in values {
        println(f"Found value {value}");
    }
    // docs:end:values_example
}

unconstrained fn iter_examples(
    mut map: UHashMap<Field, Field, BuildHasherDefault<Poseidon2Hasher>>,
) {
    // docs:start:iter_mut_example
    // Add 1 to each key in the map, and double the value associated with that key.
    map.iter_mut(|k, v| (k + 1, v * 2));
    // docs:end:iter_mut_example
    // docs:start:iter_keys_mut_example
    // Double each key, leaving the value associated with that key untouched
    map.iter_keys_mut(|k| k * 2);
    // docs:end:iter_keys_mut_example
    // docs:start:iter_values_mut_example
    // Halve each value
    map.iter_values_mut(|v| v / 2);
    // docs:end:iter_values_mut_example
}
