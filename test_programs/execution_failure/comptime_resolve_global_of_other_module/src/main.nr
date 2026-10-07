// `Expr::resolve` resolves `MAX` in `token`, so the bound is `token::MAX` (10) and not the
// crate root's `MAX`.
global MAX: u32 = 1000000;

mod token {
    global MAX: u32 = 10;

    comptime fn resolved_bound() -> Quoted {
        let e = quote { MAX }.as_expr().unwrap().resolve(Option::none());
        quote { $e }
    }

    pub fn check(x: u32) {
        assert(x <= resolved_bound!(), "x exceeds token::MAX");
    }
}

fn main(x: u32) {
    token::check(x);
    assert(MAX != 0);
}
