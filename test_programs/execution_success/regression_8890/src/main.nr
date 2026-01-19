// Regression test for issue "V-NSCA-VUL-003: Missing ArraySet case in Removing Unreachable Functions pass"
// found in Veridise Audit.

fn main(x: Field, y: pub Field) -> pub Field {
    assert(x != y);
    let mut f = [my_fun];
    f[0] = my_fun2;
    (f[0])(x)
}

fn my_fun(x: Field) -> Field {
    x + 1
}

fn my_fun2(x: Field) -> Field {
    x + 2
}
