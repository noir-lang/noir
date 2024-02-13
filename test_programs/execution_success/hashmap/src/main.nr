use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use dep::std::hash::pedersen::PedersenHasher;

struct Entry{
    key: Field,
    value: Field
}

global HASHMAP_CAP = 8;
global HASHMAP_LEN = 6;

fn allocate_hashmap() -> HashMap<Field, Field, HASHMAP_CAP, BuildHasherDefault<PedersenHasher>> {
    HashMap::default()
}

fn main(input: [Entry; HASHMAP_LEN]) {
    test_sequential(input[0].key, input[0].value);
    test_multiple_equal_insert(input[1].key, input[1].value);
    test_value_override(input[2].key, input[2].value, input[3].value);
    test_insert_and_methods(input);
    test_hashmaps_equality(input);
    test_retain();
}

// Insert, get, remove.
fn test_sequential(key: Field, value: Field) {
    let mut hashmap = allocate_hashmap();
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
fn test_multiple_equal_insert(key: Field, value: Field) {
    let mut hashmap = allocate_hashmap();
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
fn test_value_override(key: Field, value: Field, new_value: Field) {
    let mut hashmap = allocate_hashmap();
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
    let mut hashmap = allocate_hashmap();
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
    let mut hashmap = allocate_hashmap();
    assert(hashmap.is_empty(), "New HashMap should be empty.");

    let (key, value) = (5, 11);
    hashmap.insert(key, value);
    let (key, value) = (2, 13);
    hashmap.insert(key, value);
    let (key, value) = (11, 5);
    hashmap.insert(key, value);

    let predicate = |key: Field, value: Field| -> bool {key * value == 55};
    hashmap.retain(predicate);

    assert(hashmap.len() == 2, "HashMap should have retained 2 elements.");
    assert(hashmap.get(2).is_none(), "Pair should have been removed, since it does not match predicate.");
}

// Equality trait check.
fn test_hashmaps_equality(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap_1 = allocate_hashmap();
    let mut hashmap_2 = allocate_hashmap();

    for entry in input {
        hashmap_1.insert(entry.key, entry.value);
        hashmap_2.insert(entry.key, entry.value);
    }

    assert(hashmap_1 == hashmap_2, "HashMaps should be equal.");

    hashmap_2.remove(input[0].key);

    assert(hashmap_1 != hashmap_2, "HashMaps should not be equal.");
}