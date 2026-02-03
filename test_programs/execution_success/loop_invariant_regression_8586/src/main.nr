// Regression for issue #8586 (https://github.com/noir-lang/noir/issues/8586)
fn main(a: pub bool) {
    for _ in 0..1 {
        if a {
            for _ in 0..1 {
                let _ = (1 / (a as Field));
            }
        };
    }
}
