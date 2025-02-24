fn main(x: Field) -> pub Field {
    // Regression test for issue #547
    // Warning: it must be kept at the start of main
    let arr: [u8; 2] = [1, 2];
    if arr[0] != arr[1] {
        for i in 0..1 {
            assert(i != 2);
        }
    }
    // Regression for predicate simplification
    x + safe_inverse(0)
}

fn safe_inverse(n: Field) -> Field {
    if n == 0 {
        0
    } else {
        1 / n
    }
}
