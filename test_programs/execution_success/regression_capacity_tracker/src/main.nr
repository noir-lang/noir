// Reference https://github.com/noir-lang/noir/issues/4395#issuecomment-2018948631
// for context.
// We were not accurately accounting for situations where the list capacity tracker
// was expecting a capacity from list intrinsic results.
fn main(expected: pub Field, first: Field, input: [Field; 20]) {
    let mut hasher_list = input.as_list();
    hasher_list = hasher_list.push_front(first);
    assert(hasher_list[0] == expected);
    // We need a conditional based upon witnesses
    // to force a store of the list.
    // If this successfully compiles it means we have stored
    // the results of the list intrinsics used above.
    if expected as u32 > 10 {
        hasher_list[(expected - 10) as u32] = 100;
    } else {
        hasher_list[expected as u32] = 100;
    }
    assert(hasher_list[0] == expected);
}
