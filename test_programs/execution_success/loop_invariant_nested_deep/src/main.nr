// We expect this program to fail if we incorrectly hoist the division.
// This is an expansion of the `loop_invariant_regression_8586` test.
// a = false
fn main(a: pub bool) {
    for _ in 0..1 {
        for _ in 0..1 {
            for _ in 0..1 {
                for _ in 0..1 {
                    if a {
                        for _ in 0..1 {
                            let _ = (1 / (a as Field));
                        }
                    };
                }
            }
        }
    }

    for _ in 0..1 {
        if a {
            for _ in 0..1 {
                let _ = (1 / (a as Field));

                for _ in 0..1 {
                    for _ in 0..1 {
                        for _ in 0..1 {
                            let _ = (1 / (a as Field));
                        }
                    }
                }
            }
        }
    }

    for _ in 0..1 {
        for _ in 0..1 {
            for _ in 0..1 {
                if a {
                    for _ in 0..1 {
                        let _ = (1 / (a as Field));

                        for _ in 0..1 {
                            let _ = (1 / (a as Field));
                        }
                    }
                }
            }
        }
    }
}
