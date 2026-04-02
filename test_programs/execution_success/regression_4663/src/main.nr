use std::collections::umap::UHashMap;
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
    let map: UHashMap<u64, u64, BuildHasherDefault<DummyHasher>> = UHashMap::default();
    println(map == map);
}
