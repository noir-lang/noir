fn main() {
    // Safety:
    let result = unsafe { func_1([true], [false]) };
    assert_eq(result[0], true);
}
unconstrained fn func_1(mut b: [bool; 1], mut c: [bool; 1]) -> [bool; 1] {
    b[0] = {
        c = if (c[0]) {
            b[0] = true;
            b
        } else { b };
        !c[0]
    };
    c
}
