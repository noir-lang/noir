use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use dep::std::hash::poseidon2::Poseidon2Hasher;

struct Entry{
    key: Field,
    value: Field
}

global HASHMAP_CAP = 8;
global HASHMAP_LEN = 6;

fn allocate_hashmap() -> HashMap<Field, Field, HASHMAP_CAP, BuildHasherDefault<Poseidon2Hasher>> {
    HashMap::default()
}

fn main(input: [Entry; HASHMAP_LEN]) {
    test_load_factor(input);
}

// In this test we exceed load factor:
// α_max = 0.75, thus for capacity of 8 and lenght of 6
// insertion of new unique key (7-th) should throw assertion error.
fn test_load_factor(input: [Entry; HASHMAP_LEN]) {
    let mut hashmap = allocate_hashmap();

    for entry in input {
        hashmap.insert(entry.key, entry.value);
    }

    // We use prime numbers for testing, 
    // therefore it is guaranteed that doubling key we get unique value.
    let key = input[0].key * 2;
    hashmap.insert(key, input[0].value);
}
