use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use dep::std::hash::pedersen::PedersenHasher;

fn main() {
    test_sequential();
    test_multiple_equal_insert();
    test_value_override();
    test_insert_and_methods();
    test_retain();
    test_hashmaps_equality();
}

// Insert, get, remove.
fn test_sequential() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.is_empty(), "new hashmap should be empty");

    let (key, value): (Field, Field) = (1, 2);

    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "hashmap after one insert should have a length of 1 element");
    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_some(), "retrived value is none");
    let retrieved_value = retrieved_value.unwrap_unchecked();
    assert(value == retrieved_value, "retrieved value does not match inserted");
    hashmap.remove(key);
    assert(hashmap.is_empty(), "hashmap after one insert and corresponding removal should be empty");
    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_none(), "value has been removed, but is still available");
}

// Insert same pair several times.
fn test_multiple_equal_insert() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.is_empty(), "new hashmap should be empty");

    let (key, value): (Field, Field) = (1, 2);
    hashmap.insert(key, value);
    hashmap.insert(key, value);
    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "hashmap length is invalid");

    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_some(), "retrieved value is none");
    let retrieved_value = retrieved_value.unwrap_unchecked();
    assert(retrieved_value == value, "retrieved value is not correct");
}

// Override value for existing pair.
fn test_value_override() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.is_empty(), "new hashmap should be empty");

    let (key, value): (Field, Field) = (1, 2);
    hashmap.insert(key, value);
    let value = 3;
    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "hashmap length is invalid");

    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_some(), "retrieved value is none");
    let retrieved_value = retrieved_value.unwrap_unchecked();
    assert(retrieved_value == 3, "retrieved value is not correct");
}

// Insert several distinct pairs and test auxiliary methods.
fn test_insert_and_methods() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.is_empty(), "new hashmap should be empty");

    let (key, value) = (5, 11);
    hashmap.insert(key, value);
    let (key, value) = (2, 13);
    hashmap.insert(key, value);
    let (key, value) = (3, 7);
    hashmap.insert(key, value);

    assert(hashmap.len() == 3, "hashmap length should be 3");

    assert(hashmap.contains_key(5), "hashmap does not contain key 5");
    assert(hashmap.contains_key(2), "hashmap does not contain key 2");
    assert(hashmap.contains_key(3), "hashmap does not contain key 3");

    let mut keys = hashmap.keys();
    let mut accumulator = 1;
    for _ in 0..3 {
        let key = keys.pop();
        assert((key == 5) | (key == 2) | (key == 3), "found unexpected key");
        accumulator *= key;
    }
    assert(accumulator == 30, "got invalid set of keys");

    let mut values = hashmap.values();
    let mut accumulator = 1;
    for _ in 0..3 {
        let value = values.pop();
        assert((value == 11) | (value == 13) | (value == 7), "found unexpected value");
        accumulator *= value;
    }
    assert(accumulator == 1001, "got invalid set of values");

    hashmap.clear();
    assert(hashmap.is_empty(), "hashmap after clear should be empty");
}

// Insert several pairs and test retaining.
fn test_retain() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.is_empty(), "new hashmap should be empty");

    let (key, value) = (5, 11);
    hashmap.insert(key, value);
    let (key, value) = (2, 13);
    hashmap.insert(key, value);
    let (key, value) = (11, 5);
    hashmap.insert(key, value);

    let predicate = |key: Field, value: Field| -> bool {key * value == 55};
    hashmap.retain(predicate);

    assert(hashmap.len() == 2, "HashMap should have retained 2 elements");
    assert(hashmap.get(2).is_none(), "pair should have been removed, since it does not match predicate");
}

// Equality trait check.
fn test_hashmaps_equality() {
    let mut hashmap_1: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    let mut hashmap_2: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();

    let (key, value) = (5, 11);
    hashmap_1.insert(key, value);
    let (key, value) = (2, 13);
    hashmap_1.insert(key, value);
    let (key, value) = (11, 5);
    hashmap_1.insert(key, value);

    let (key, value) = (5, 11);
    hashmap_2.insert(key, value);
    let (key, value) = (2, 13);
    hashmap_2.insert(key, value);
    let (key, value) = (11, 5);
    hashmap_2.insert(key, value);

    assert(hashmap_1 == hashmap_2, "HashMaps should be equal");

    hashmap_2.remove(11);

    assert(hashmap_1 != hashmap_2, "HashMaps should not be equal");
}
