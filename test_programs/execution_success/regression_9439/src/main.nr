unconstrained fn main(n: u32) {
    let num1 = &mut 0;
    let mut slice = &[];
    for _ in 0..n {
        slice = slice.push_back(num1);
    }
    *slice[0] = 9;
    assert_eq(*slice[0], 9);
    assert_eq(*num1, 9);
}
