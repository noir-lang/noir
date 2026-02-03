// Reference https://github.com/noir-lang/noir/issues/4395#issuecomment-2018948631
// for context.
// We were not accurately accounting for situations where the vector capacity tracker
// was expecting a capacity from vector intrinsic results.
fn main(expected: pub Field, first: Field, input: [Field; 20]) {
    let mut hasher_vector = input.as_vector();
    hasher_vector = hasher_vector.push_front(first);
    assert(hasher_vector[0] == expected);
    // We need a conditional based upon witnesses
    // to force a store of the vector.
    // If this successfully compiles it means we have stored
    // the results of the vector intrinsics used above.
    if expected as u32 > 10 {
        hasher_vector[(expected - 10) as u32] = 100;
    } else {
        hasher_vector[expected as u32] = 100;
    }
    assert(hasher_vector[0] == expected);
}
