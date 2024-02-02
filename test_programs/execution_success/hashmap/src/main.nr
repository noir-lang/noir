use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use dep::std::hash::pedersen::PedersenHasher;

fn main() {
    test_sequential();
    test_multiple_same_insert();
    test_value_override();
}

// Insert, get, remove.
fn test_sequential() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.len() == 0, "new hashmap should be empty");

    let (key, value): (Field, Field) = (1, 2);

    hashmap.insert(key, value);
    assert(hashmap.len() == 1, "hashmap after one insert should have a length of 1 element");

    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_some(), "retrived value is none");

    let retrieved_value = retrieved_value.unwrap_unchecked();
    assert(value == retrieved_value, "retrieved value does not match inserted");

    hashmap.remove(key);
    assert(
        hashmap.len() == 0, "hashmap after one insert and corresponding removal should have a lenght of 1 element"
    );

    let retrieved_value = hashmap.get(key);
    assert(retrieved_value.is_none(), "value has been removed, but is still available");
}

// Insert same pair several times.
fn test_multiple_same_insert() {
    let mut hashmap: HashMap<Field, Field, 4, BuildHasherDefault<PedersenHasher>> = HashMap::default();
    assert(hashmap.len() == 0, "new hashmap should be empty");

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
    assert(hashmap.len() == 0, "new hashmap should be empty");

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
