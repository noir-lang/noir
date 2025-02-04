fn main() {
    /// Safety: test code
    unsafe {
        check_while();
    }

    check_comptime_while();
}

unconstrained fn check_while() {
    let mut i = 0;
    let mut sum = 0;

    while i < 4 {
        if i == 2 {
            i += 1;
            continue;
        }

        sum += i;
        i += 1;
    }

    assert_eq(sum, 1 + 3);
}

fn check_comptime_while() {
    comptime {
        let mut i = 0;
        let mut sum = 0;

        while i < 4 {
            if i == 2 {
                i += 1;
                continue;
            }

            sum += i;
            i += 1;
        }

        assert_eq(sum, 1 + 3);
    }
}
