// Reference https://github.com/noir-lang/noir/issues/4395#issuecomment-2018948631
// for context.
// We were not accurately accounting for situations where the slice capacity tracker
// was expecting a capacity from slice intrinsic results.
fn main(expected: pub Field, first: Field, input: [Field; 20]) {
    let mut hasher_slice = input.as_slice();
    hasher_slice = hasher_slice.push_front(first);
    assert(hasher_slice[0] == expected);
    // We need a conditional based upon witnesses
    // to force a store of the slice.
    // If this successfully compiles it means we have stored
    // the results of the slice intrinsics used above.
    if expected as u32 > 10 {
        hasher_slice[expected - 10] = 100;
    } else {
        hasher_slice[expected] = 100;
    }
    assert(hasher_slice[0] == expected);
}
