// Regression for issue #5545 (https://github.com/noir-lang/noir/issues/5545)
fn main() {
    #[allow(unused_mut)]
    let mut c = || {
        #[allow(unused_mut)]
        let mut d = || {};
        d();
    };

    c();
}
