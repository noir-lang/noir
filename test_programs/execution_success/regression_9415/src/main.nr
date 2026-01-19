unconstrained fn main(index: u32) {
    let v0 = &mut 1;
    let v1 = &mut 2;
    let mut v3 = [v0];
    v3[index] = v1;
    *v3[index] = 3;
    assert_eq(*v3[index], 3);
}
