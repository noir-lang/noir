// Merging two vectors of different lengths and then conditionally writing a constant
// index used to make `array_set_window_optimization` read past the shorter branch's
// length (it reconstructed the merged vector up to the larger branch's capacity),
// producing intermediate SSA that failed interpretation with an out-of-bounds error.
fn branch_merge_case(choose: bool, cond: bool) -> Field {
    let mut vec: [Field] = if choose {
        [1, 2, 3].as_vector()
    } else {
        [4, 5, 6, 7].as_vector()
    };
    if cond {
        vec[1] = 99;
    }
    vec[2]
}

fn main(choose: bool, cond: bool) -> pub Field {
    branch_merge_case(choose, cond)
}
