use std::hash::poseidon2;

fn main() {
    let mut hasher_1 = poseidon2::Poseidon2Hasher::default();

    for _ in 0..128 {
        hasher_1.write(42);
    }
    hasher_1.write(1);

    let mut hasher_2 = poseidon2::Poseidon2Hasher { _state: hasher_1._state };
    for _ in 0..128 {
        hasher_2.write(42);
    }
    let result_1 = hasher_1.finish();
    hasher_2.write(2);
    let result_2 = hasher_2.finish();

    let mut hasher_3 = poseidon2::Poseidon2Hasher::default();
    for _ in 0..128 {
        hasher_3.write(42);
    }
    hasher_3.write(1);
    for _ in 0..128 {
        hasher_3.write(42);
    }
    hasher_3.write(2);
    let result_3 = hasher_3.finish();

    assert(result_2 == result_3);
    println(result_1);
    println(result_2);
    println(result_3);
}

