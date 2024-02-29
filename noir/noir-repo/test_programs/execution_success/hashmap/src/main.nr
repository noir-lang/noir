mod utils;

use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use dep::std::hash::pedersen::PedersenHasher;
use dep::std::cmp::Eq;

use utils::cut;

type K = Field;
type V = Field;

// It is more convenient and readable to use structs as input.
struct Entry{
    key: Field,
    value: Field
}

global HASHMAP_CAP = 8;
global HASHMAP_LEN = 6;

global FIELD_CMP = |a: Field, b: Field| a.lt(b);

global K_CMP = FIELD_CMP;
global V_CMP = FIELD_CMP;
global KV_CMP = |a: (K, V), b: (K, V)| a.0.lt(b.0);

global ALLOCATE_HASHMAP = || -> HashMap<K, V, HASHMAP_CAP, BuildHasherDefault<PedersenHasher>> 
    HashMap::default();

fn main(input: [Entry; HASHMAP_LEN]) {
    test_sequential(input[0].key, input[0].value);
    test_multiple_equal_insert(input[1].key, input[1].value);
    test_value_override(input[2].key, input[2].value, input[3].value);
    test_insert_and_methods(input);
    test_hashmaps_equality(input);
    test_retain();
    test_iterators();
    test_mut_iterators();
}

// Insert, get, remove.
fn test_sequential(key: K, value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New HashMap should be empty.");

    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "HashMap after one insert should have a length of 1 element.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(value == got, f"Inserted {value} but got {got} for the same key.");

    hashmap.remove(key);
    assert(hashmap.is_empty(), "HashMap after one insert and corresponding removal should be empty.");
    let got = hashmap.get(key);
    assert(got.is_none(), "Value has been removed, but is still available (not none).");
}

// Insert same pair several times.
fn test_multiple_equal_insert(key: K, value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New HashMap should be empty.");

    for _ in 0..HASHMAP_LEN {
        hashmap.insert(key, value);
    }

    let len = hashmap.len();
    assert(len == 1, f"HashMap length must be 1, got {len}.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(value == got, f"Inserted {value} but got {got} for the same key.");
}

// Override value for existing pair.
fn test_value_override(key: K, value: V, new_value: V) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New hashmap should be empty.");

    hashmap.insert(key, value);
    hashmap.insert(key, new_value);
    assert(hashmap.len() == 1, "HashMap length is invalid.");

    let got = hashmap.get(key);
    assert(got.is_some(), "Got none value.");
    let got = got.unwrap_unchecked();
    assert(got == new_value, f"Expected {new_value}, but got {got}.");
}

// Insert several distinct pairs and test auxiliary methods.
fn test_insert_and_methods(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New HashMap should be empty.");

    for entry in input {
        hashmap.insert(entry.key, entry.value);
    }

    assert(hashmap.len() == HASHMAP_LEN, "hashmap.len() does not match input lenght.");

    for entry in input {
        assert(hashmap.contains_key(entry.key), f"Not found inserted key {entry.key}.");
    }

    hashmap.clear();
    assert(hashmap.is_empty(), "HashMap after clear() should be empty.");
}

// Insert several pairs and test retaining.
fn test_retain() {
    let mut hashmap = ALLOCATE_HASHMAP();
    assert(hashmap.is_empty(), "New HashMap should be empty.");

    let (key, value) = (5, 11);
    hashmap.insert(key, value);
    let (key, value) = (2, 13);
    hashmap.insert(key, value);
    let (key, value) = (11, 5);
    hashmap.insert(key, value);

    let predicate = |key: K, value: V| -> bool {key * value == 55};
    hashmap.retain(predicate);

    assert(hashmap.len() == 2, "HashMap should have retained 2 elements.");
    assert(hashmap.get(2).is_none(), "Pair should have been removed, since it does not match predicate.");
}

// Equality trait check.
fn test_hashmaps_equality(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap_1 = ALLOCATE_HASHMAP();
    let mut hashmap_2 = ALLOCATE_HASHMAP();

    for entry in input {
        hashmap_1.insert(entry.key, entry.value);
        hashmap_2.insert(entry.key, entry.value);
    }

    assert(hashmap_1 == hashmap_2, "HashMaps should be equal.");

    hashmap_2.remove(input[0].key);

    assert(hashmap_1 != hashmap_2, "HashMaps should not be equal.");
}

// Test entries, keys, values.
fn test_iterators() {
    let mut hashmap = ALLOCATE_HASHMAP();

    hashmap.insert(2, 3);
    hashmap.insert(5, 7);
    hashmap.insert(11, 13);

    let keys: [K; 3] = cut(hashmap.keys()).map(|k: Option<K>| k.unwrap_unchecked()).sort_via(K_CMP);
    let values: [V; 3] = cut(hashmap.values()).map(|v: Option<V>| v.unwrap_unchecked()).sort_via(V_CMP);
    let entries: [(K, V); 3] = cut(hashmap.entries()).map(|e: Option<(K, V)>| e.unwrap_unchecked()).sort_via(KV_CMP);

    assert(keys == [2, 5, 11], "Got incorrect iteration of keys.");
    assert(values == [3, 7, 13], "Got incorrect iteration of values.");
    assert(entries == [(2, 3), (5, 7), (11, 13)], "Got incorrect iteration of entries.");
}

// Test mutable iteration over keys, values and entries.
fn test_mut_iterators() {
    let mut hashmap = ALLOCATE_HASHMAP();

    hashmap.insert(2, 3);
    hashmap.insert(5, 7);
    hashmap.insert(11, 13);

    let f = |k: K| -> K{ k * 3};
    hashmap.iter_keys_mut(f);

    let f = |v: V| -> V{ v * 5};
    hashmap.iter_values_mut(f);

    let keys: [K; 3] = cut(hashmap.keys()).map(|k: Option<K>| k.unwrap_unchecked()).sort_via(K_CMP);
    let values: [V; 3] = cut(hashmap.values()).map(|v: Option<V>| v.unwrap_unchecked()).sort_via(V_CMP);

    assert(keys == [6, 15, 33], f"Got incorrect iteration of keys: {keys}");
    assert(values == [15, 35, 65], "Got incorrect iteration of values.");

    let f = |k: K, v: V| -> (K, V){(k * 2, v * 2)};
    hashmap.iter_mut(f);

    let entries: [(K, V); 3] = cut(hashmap.entries()).map(|e: Option<(K, V)>| e.unwrap_unchecked()).sort_via(KV_CMP);

    assert(entries == [(12, 30), (30, 70), (66, 130)], "Got incorrect iteration of entries.");
}
