fn main() {
    let result1 = bug_8262((1, true, false));
    assert_eq(result1, 2);

    let result2 = bug_8337();
    assert_eq(result2, [10, 40, 10]);
}

fn bug_8262(mut a: (i32, bool, bool)) -> i32 {
    a.1 = if a.1 {
        a = (2, a.2, a.1);
        true
    } else { !a.2 };
    a.0
}

fn bug_8337() -> [Field; 3] {
    let mut a = [10, 20, 30];
    a[1] = {
        a = {
            a[2] = a[0];
            [a[0], 40, a[2]]
        };
        a[1]
    };
    a
}
