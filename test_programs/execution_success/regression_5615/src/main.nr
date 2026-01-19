use poseidon::poseidon2::Poseidon2Hasher;
use std::collections::umap::UHashMap;
use std::hash::BuildHasherDefault;

unconstrained fn main() {
    comptime {
        let mut map: UHashMap<u32, Field, BuildHasherDefault<Poseidon2Hasher>> =
            UHashMap::default();

        map.insert(1, 2);
    }
}
