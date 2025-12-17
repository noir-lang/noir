unconstrained fn main(n: u32) {
    let num1 = &mut 0;
    let mut list = &[];
    for _ in 0..n {
        list = list.push_back(num1);
    }
    *list[0] = 9;
    assert_eq(*list[0], 9);
    assert_eq(*num1, 9);
}
