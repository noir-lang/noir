// Passing a generic foreign function (`blake2s`) as a value at two different
// monomorphized types must produce a distinct proxy per instantiation. A shared
// proxy cache keyed only on the foreign name reused the first instantiation's
// signature, silently constraining the second hash to zeros.
fn call_one(f: fn([u8; 1]) -> [u8; 32]) -> [u8; 32] {
    f([1])
}

fn call_two(f: fn([u8; 2]) -> [u8; 32]) -> [u8; 32] {
    f([2, 3])
}

fn main() {
    let a = call_one(std::hash::blake2s);
    let b = call_two(std::hash::blake2s);
    assert(a == std::hash::blake2s([1]));
    assert(b == std::hash::blake2s([2, 3]));
}
