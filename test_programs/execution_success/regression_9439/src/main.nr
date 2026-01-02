unconstrained fn main(n: u32) {
    let num1 = &mut 0;
    let mut vector = &[];
    for _ in 0..n {
        vector = vector.push_back(num1);
    }
    *vector[0] = 9;
    assert_eq(*vector[0], 9);
    assert_eq(*num1, 9);
}
