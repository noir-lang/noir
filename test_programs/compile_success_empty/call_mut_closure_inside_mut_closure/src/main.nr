// Regression for issue #5545 (https://github.com/noir-lang/noir/issues/5545)
fn main() {
    let mut c = || {
        let mut d = || {};
        d();
    };

    c();
}
