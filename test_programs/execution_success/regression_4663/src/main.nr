use dep::std::collections::map::HashMap;
use dep::std::hash::BuildHasherDefault;
use poseidon::poseidon::PoseidonHasher;

fn main() {
    let map: HashMap<u64, u64, 10, BuildHasherDefault<PoseidonHasher>> = HashMap::default();
    println(map == map);
}
