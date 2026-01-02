use std::collections::map::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

struct DummyHasher {}

impl Hasher for DummyHasher {
    fn finish(self) -> Field {
        0
    }

    fn write(&mut self, _input: Field) {}
}

impl Default for DummyHasher {
    fn default() -> Self {
        DummyHasher {}
    }
}

fn main() {
    let map: HashMap<u64, u64, 10, BuildHasherDefault<DummyHasher>> = HashMap::default();
    println(map == map);
}
