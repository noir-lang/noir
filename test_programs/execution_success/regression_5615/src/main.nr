use std::{collections::umap::UHashMap, hash::{BuildHasherDefault, poseidon2::Poseidon2Hasher}};

unconstrained fn main() {
    comptime {
        let mut map: UHashMap<u32, Field, BuildHasherDefault<Poseidon2Hasher>> =
            UHashMap::default();

        map.insert(1, 2);
    }
}
