unconstrained fn f(input: u32) -> [u8; 2] {
    [input as u8; 2]
}
fn main(input: u32, expected_sum: pub u32) {
    // safety: test
    let data = unsafe { f(input) };
    let mut sum: u32 = 0;
    for i in 0..2 {
        sum += data[i] as u32;
    }
    // This constraint does not connect the inputs and outputs of the call to `f` above,
    // so `check_for_missing_brillig_constraints` does not consider it sufficient.
    assert(sum == expected_sum);
    // This constraint on the output against a constant would be considered sufficient
    // by `check_for_missing_brillig_constraints`, however the 'Constant Folding using constraints'
    // pass turned this into `assert(expected_sum < 1000000)` due to the previous constraint.
    assert(sum < 1000000);
}
