unconstrained fn main() {
    let mut arr = [1, 2, 3];
    let mut i = 0;
    // `check` must never be inlined for this regression test to be meaningful
    while check(arr) {
        i += 1;
        if i > 10 {
            break;
        }
    }
    // This assertion acts as a regression that we do not exit the while loop early
    // due to an invalid shared mutation inside of `check`
    assert_eq(i, 11);
}

fn check(mut a: [Field; 3]) -> bool {
    let res = a[0] == 1;
    a[0] = 50;
    res
}
