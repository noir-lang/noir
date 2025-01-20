fn main() {
    /// Safety: test code
    unsafe {
        check_loop();
    }

    check_comptime_loop();
}

unconstrained fn check_loop() {
    let mut i = 0;
    let mut sum = 0;

    loop {
        if i == 4 {
            break;
        }

        if i == 2 {
            i += 1;
            continue;
        }

        sum += i;
        i += 1;
    }

    assert_eq(sum, 1 + 3);
}

fn check_comptime_loop() {
    comptime {
        let mut i = 0;
        let mut sum = 0;

        loop {
            if i == 4 {
                break;
            }

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
